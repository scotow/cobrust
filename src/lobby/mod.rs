pub mod packet;

use std::collections::HashMap;
use crate::game::Game;
use warp::ws::{WebSocket, Message};
use futures::{StreamExt, SinkExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use futures::stream::SplitSink;
use tokio::task;
use futures::future::join_all;
use crate::lobby::packet::Packet;
use crate::game::config::Config;

pub struct Lobby {
    inner: Arc<Mutex<Inner>>,
    // games: Arc<Mutex<HashMap<u16, Arc<Game>>>>,
    // users: Arc<Mutex<HashMap<u16, Arc<Mutex<SplitSink<WebSocket, Message>>>>>>,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                games: HashMap::new(),
                users: HashMap::new(),
            }))
        }
    }

    pub async fn join(&self, socket: WebSocket) {
        let id = rand::random();
        let (mut tx, mut rx) = socket.split();

        let mut inner = self.inner.lock().await;
        let games_message = Packet::AddGames(
            inner.games.iter().collect()
        ).message().await;
        tx.send(games_message).await.unwrap();

        let tx = Arc::new(Mutex::new(tx));
        inner.users.insert(id, Arc::clone(&tx));
        drop(inner);

        loop {
            let message = match rx.next().await {
                Some(Ok(message)) => message,
                _ => break,
            };
            if message.is_close() {
                break;
            }

            let data = message.as_bytes();
            match data[0] {
                0 => {
                    let mut inner = self.inner.lock().await;
                    let (id, game) = match inner.create(&data[1..]) {
                        Some(res) => res,
                        None => continue,
                    };

                    tx.lock().await.send(Packet::GameCreated(id).message().await).await.unwrap();
                    inner.broadcast_message(
                        Packet::AddGames(vec![(&id, &game)]).message().await
                    ).await;

                    let inner_ref = Arc::clone(&self.inner);
                    task::spawn(async move {
                        game.run().await;
                        // Game is over from here.

                        let mut inner = inner_ref.lock().await;
                        inner.broadcast_message(Packet::RemoveGame(id).message().await).await;
                        inner.games.remove(&id);
                    });
                },
                _ => (),
            }
        }
        self.inner.lock().await.users.remove(&id);
    }

    pub async fn play(&self, id: u16, socket: WebSocket) {
        let mut inner = self.inner.lock().await;
        let game = Arc::clone(inner.games.get(&id).unwrap());
        inner.broadcast_message(
            Packet::PlayerCount(id, (game.player_count().await + 1) as u8).message().await
        ).await;
        drop(inner);

        game.play(socket).await;
        self.inner.lock().await.broadcast_message(
            Packet::PlayerCount(id, game.player_count().await as u8).message().await
        ).await;
    }
}

struct Inner {
    games: HashMap<u16, Arc<Game>>,
    users: HashMap<u16, Arc<Mutex<SplitSink<WebSocket, Message>>>>,
}

impl Inner {
    pub fn create(&mut self, data: &[u8]) -> Option<(u16, Arc<Game>)> {
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
        join_all(self.users.values_mut()
            .map(|user| {
                let message = message.clone();
                async move {
                    let _ = user.lock().await.send(message).await;
                }
            })
        ).await;
    }
}