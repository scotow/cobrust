use std::sync::Arc;

use axum::extract::ws::Message;

use crate::{game::Game, lobby::GameId, misc::ToData, packet};

pub enum Packet<'a> {
    AddGames(Vec<(&'a GameId, &'a Arc<Game>)>),
    RemoveGame(GameId),
    PlayerCount(GameId, u8),
    GameCreated(GameId),
}

impl<'a> Packet<'a> {
    pub async fn message(self) -> Message {
        let payload = match self {
            Packet::AddGames(games) => {
                let mut packet = Vec::with_capacity(128);
                packet.push(0);
                for (&id, game) in games {
                    packet![packet; id,
                        game.name.as_bytes().len() as u8, game.name.as_bytes(),
                        game.size.width as u16, game.size.height as u16,
                        game.speed,
                        game.player_count().await as u8
                    ];
                }
                packet
            }
            Packet::RemoveGame(id) => {
                packet![1u8, id]
            }
            Packet::PlayerCount(id, count) => {
                packet![2u8, id, count]
            }
            Packet::GameCreated(id) => {
                packet![3u8, id]
            }
        };
        Message::Binary(payload)
    }
}
