use warp::Filter;
use std::sync::Arc;
use warp::ws::WebSocket;
use std::convert::TryInto;
use crate::lobby::Lobby;

mod game;
mod player;
mod coordinate;
mod direction;
mod size;
mod perk;
mod cell;
mod packet;
mod lobby;

#[tokio::main]
async fn main() {
    env_logger::init();

    let [_width, _height]: [usize; 2] = std::env::args().skip(1)
        .map(|s| s.parse::<usize>().unwrap())
        .collect::<Vec<_>>()
        .try_into().unwrap();

    let lobby = Arc::new(Lobby::new());
    let lobby_ref = warp::any().map(move || Arc::clone(&lobby));
    let lobby_route = warp::path("lobby")
        .and(warp::ws())
        .and(lobby_ref.clone())
        .map(|websocket: warp::ws::Ws, lobby: Arc<Lobby>| {
            websocket.on_upgrade(move |socket| join_lobby(lobby, socket))
        });
    let game_route = warp::path!("games" / u16)
        .and(warp::ws())
        .and(lobby_ref.clone())
        .map(|id, websocket: warp::ws::Ws, lobby: Arc<Lobby>| {
            websocket.on_upgrade(move |socket| join_game(lobby, id, socket))
        });

    warp::serve(
        warp::fs::dir("src/public")
            .or(lobby_route)
            .or(game_route)
    ).run(([0, 0, 0, 0], 3030)).await;
}

async fn join_lobby(lobby: Arc<Lobby>, socket: WebSocket) {
    lobby.join(socket).await;
}

async fn join_game(lobby: Arc<Lobby>, id: u16, socket: WebSocket) {
    lobby.play(id, socket).await;
}