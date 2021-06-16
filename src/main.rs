use tokio::task;
use warp::Filter;

use crate::game::Game;
use std::sync::Arc;
use warp::ws::WebSocket;

mod game;
mod player;
mod coordinate;
mod direction;
mod size;
mod perk;
mod cell;

#[tokio::main]
async fn main() {
    env_logger::init();

    let game = Arc::new(Game::new());

    let game_loop = Arc::clone(&game);
    task::spawn(async move {
        game_loop.run().await;
    });

    let ws = warp::path("ws")
        .and(warp::any().map(move || Arc::clone(&game)))
        .and(warp::ws())
        .map(|game: Arc<Game>, websocket: warp::ws::Ws| {
            websocket.on_upgrade(move |socket| user_connected(game, socket))
        });

    warp::serve(warp::fs::dir("src/public").or(ws)).run(([0, 0, 0, 0], 3030)).await;
}

async fn user_connected(game: Arc<Game>, socket: WebSocket) {
    game.add_player(socket).await;
}