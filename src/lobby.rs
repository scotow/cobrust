use std::collections::HashMap;
use crate::game::Game;
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
    users: Mutex<HashMap<u16, SplitSink<WebSocket, Message>>>,
}

impl Lobby {
    pub fn new() -> Self {
        let game = Arc::new(Game::new(
            String::from("Public Game"),
            Size {
                width: 32,
                height: 32,
            }));
        let mut games = HashMap::new();
        games.insert(42, Arc::clone(&game));

        task::spawn(async move {
            game.run().await;
        });

        Self {
            games: Mutex::new(games),
            users: Mutex::new(HashMap::new()),
        }
    }

    pub async fn join(&self, socket: WebSocket) {
        let id = rand::random();
        let (mut tx, mut rx) = socket.split();

        let games_messages = Packet::Games(&*self.games.lock().await).message().await;
        tx.send(games_messages).await.unwrap();

        self.users.lock().await.insert(id, tx);
        loop {
            let message = match rx.next().await {
                Some(Ok(message)) => message,
                _ => break,
            };
            if message.is_close() {
                break;
            }
        }
        self.users.lock().await.remove(&id);
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

    // async fn move_player(&self, game_id: u16, user_id: u16, tx: Arc<Mutex<SplitSink<WebSocket, Message>>>, rx: &mut SplitStream<WebSocket>) {
    //     let game = Arc::clone(&self.games.lock().await.get(&game_id).unwrap());
    //     self.broadcast_message(Packet::GamePlayerCount(
    //         game_id,
    //         (game.player_count().await + 1) as u8
    //     ).message().await).await;
    //
    //     let intended_exit = game.play(user_id, tx, rx).await;
    //     if !intended_exit {
    //         self.users.lock().await.remove(&user_id);
    //     }
    //     self.broadcast_message(
    //         Packet::GamePlayerCount(game_id, game.player_count().await as u8).message().await
    //     ).await;
    //
    //     intended_exit
    // }

    async fn broadcast_message(&self, message: Message) {
        join_all(self.users.lock().await.values_mut()
            .map(|user| {
                let message = message.clone();
                async move {
                    let _ = user.send(message).await;
                }
            })
        ).await;
    }
}