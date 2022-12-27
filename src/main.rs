use std::{error::Error as StdError, net::SocketAddr, sync::Arc, u16};

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router, Server,
};

use crate::lobby::Lobby;

mod asset;
mod game;
mod lobby;
mod misc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError + Send + Sync>> {
    env_logger::init();

    let router = Router::new()
        .route("/lobby", get(lobby_handler))
        .route("/games/:id", get(join_game_handler))
        .with_state(Arc::new(Lobby::new()))
        .route("/", get(asset::handler))
        .route("/:asset", get(asset::handler));

    Server::bind(&SocketAddr::new("0.0.0.0".parse()?, 8080))
        .http1_title_case_headers(true)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}

async fn lobby_handler(State(lobby): State<Arc<Lobby>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| async move {
        lobby.join(socket).await;
    })
}

async fn join_game_handler(
    State(lobby): State<Arc<Lobby>>,
    Path(id): Path<u16>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        lobby.play(id, socket).await;
    })
}
