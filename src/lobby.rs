use std::collections::HashMap;
use crate::game::{Game, Config};
use warp::ws::{WebSocket, Message};
use futures::{StreamExt, SinkExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use futures::stream::SplitSink;
use crate::size::Size;
use tokio::task;
use futures::future::join_all;
use crate::packet::Packet;

pub struct Lobby {
    games: Mutex<HashMap<u16, Arc<Game>>>,
    users: Mutex<HashMap<u16, Arc<Mutex<SplitSink<WebSocket, Message>>>>>,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            games: Mutex::new(HashMap::new()),
            users: Mutex::new(HashMap::new()),
        }
    }

    pub async fn join(&self, socket: WebSocket) {
        let id = rand::random();
        let (mut tx, mut rx) = socket.split();

        let games_messages = Packet::Games(
            self.games.lock().await.iter().collect()
        ).message().await;
        tx.send(games_messages).await.unwrap();
        let tx = Arc::new(Mutex::new(tx));

        self.users.lock().await.insert(id, Arc::clone(&tx));
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
                    if let Some((id, game)) = self.create(&data[1..]).await {
                        self.broadcast_message(Packet::Games(vec![(&id, &game)]).message().await).await;
                        tx.lock().await.send(Packet::GameCreated(id).message().await).await.unwrap();
                    }
                },
                _ => (),
            }
        }
        self.users.lock().await.remove(&id);
    }

    pub async fn create(&self, data: &[u8]) -> Option<(u16, Arc<Game>)> {
        let size_name = u16::from_be_bytes([data[0], data[1]]) as usize;
        let name = String::from_utf8(data[2..(2 + size_name)].to_vec()).unwrap();
        let size = Size {
            width: data[2 + size_name] as usize,
            height: data[2 + size_name + 1] as usize,
        };
        let foods = data[2 + size_name + 1 + 1];

        let config = Config { name, size, foods };
        if !config.is_valid() {
            return None;
        }

        let id = rand::random();
        let game = Arc::new(Game::new(config));
        self.games.lock().await.insert(id, Arc::clone(&game));

        let game_loop = Arc::clone(&game);
        task::spawn(async move {
            game_loop.run().await;
        });

        Some((id, game))
    }

    pub async fn play(&self, id: u16, socket: WebSocket) {
        let game = Arc::clone(&self.games.lock().await.get(&id).unwrap());
        self.broadcast_message(Packet::GamePlayerCount(
            id,
            (game.player_count().await + 1) as u8
        ).message().await).await;

        game.play(socket).await;
        self.broadcast_message(
            Packet::GamePlayerCount(id, game.player_count().await as u8).message().await
        ).await;
    }

    async fn broadcast_message(&self, message: Message) {
        join_all(self.users.lock().await.values_mut()
            .map(|user| {
                let message = message.clone();
                async move {
                    let _ = user.lock().await.send(message).await;
                }
            })
        ).await;
    }
}