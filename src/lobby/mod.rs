use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{future::join_all, stream::SplitSink, SinkExt, StreamExt};
use packet::Packet;
use tokio::{sync::Mutex, task};

use crate::game::{config::Config, Game};

pub mod packet;

type GameId = u16;
type UserId = u16;

pub struct Lobby {
    inner: Arc<Mutex<Inner>>,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                games: HashMap::new(),
                users: HashMap::new(),
            })),
        }
    }

    pub async fn join(&self, socket: WebSocket) {
        let id = rand::random();
        let (mut tx, mut rx) = socket.split();

        let mut inner = self.inner.lock().await;
        let games_message = Packet::AddGames(inner.games.iter().collect())
            .message()
            .await;
        tx.send(games_message).await.unwrap();

        let tx = Arc::new(Mutex::new(tx));
        inner.users.insert(id, Arc::clone(&tx));
        drop(inner);

        loop {
            let message = match rx.next().await {
                Some(Ok(message)) => message,
                _ => break,
            };

            let Message::Binary(data) = message else {
                break;
            };
            match data[0] {
                0 => {
                    let mut inner = self.inner.lock().await;
                    let (id, game) = match inner.create(&data[1..]) {
                        Some(res) => res,
                        None => continue,
                    };

                    tx.lock()
                        .await
                        .send(Packet::GameCreated(id).message().await)
                        .await
                        .unwrap();
                    inner
                        .broadcast_message(Packet::AddGames(vec![(&id, &game)]).message().await)
                        .await;

                    let inner_ref = Arc::clone(&self.inner);
                    task::spawn(async move {
                        game.run().await;
                        // Game is over from here.

                        let mut inner = inner_ref.lock().await;
                        inner
                            .broadcast_message(Packet::RemoveGame(id).message().await)
                            .await;
                        inner.games.remove(&id);
                    });
                }
                _ => (),
            }
        }
        self.inner.lock().await.users.remove(&id);
    }

    pub async fn play(&self, id: GameId, socket: WebSocket) {
        let mut inner = self.inner.lock().await;
        let Some(game) = inner.games.get(&id).cloned() else {
            let _ = socket.close().await;
            return;
        };
        inner
            .broadcast_message(
                Packet::PlayerCount(id, (game.player_count().await + 1) as u8)
                    .message()
                    .await,
            )
            .await;
        drop(inner);

        game.play(socket).await;
        self.inner
            .lock()
            .await
            .broadcast_message(
                Packet::PlayerCount(id, game.player_count().await as u8)
                    .message()
                    .await,
            )
            .await;
    }
}

struct Inner {
    games: HashMap<GameId, Arc<Game>>,
    users: HashMap<UserId, Arc<Mutex<SplitSink<WebSocket, Message>>>>,
}

impl Inner {
    pub fn create(&mut self, data: &[u8]) -> Option<(GameId, Arc<Game>)> {
        let config = Config::from_raw(data)?;
        if !config.is_valid() {
            return None;
        }

        let id = rand::random();
        let game = Arc::new(Game::new(config));
        self.games.insert(id, Arc::clone(&game));

        Some((id, game))
    }

    async fn broadcast_message(&mut self, message: Message) {
        join_all(self.users.values_mut().map(|user| async {
            let _ = user.lock().await.send(message.clone()).await;
        }))
        .await;
    }
}
