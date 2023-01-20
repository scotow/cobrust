use std::sync::Arc;

use axum::extract::ws::Message;
use enum_index::EnumIndex;
use enum_index_derive::EnumIndex;

use crate::{game::Game, lobby::GameId, misc::PacketSerialize, packet};

#[derive(EnumIndex, Debug)]
pub enum Packet<'a> {
    AddGames(Vec<(&'a GameId, &'a Arc<Game>)>),
    RemoveGame(GameId),
    PlayerCount(GameId, u8),
    GameCreated(GameId),
}

impl<'a> Packet<'a> {
    pub async fn message(self) -> Message {
        let mut payload = packet![cap 128; self.enum_index()  as u8];
        match self {
            Packet::AddGames(games) => {
                for (&id, game) in games {
                    packet![payload; id,
                        game.name.as_bytes().len() as u8, game.name.as_bytes(),
                        game.size,
                        game.speed,
                        game.player_count().await as u8
                    ];
                }
            }
            Packet::RemoveGame(id) => {
                packet![payload; id]
            }
            Packet::PlayerCount(id, count) => {
                packet![payload; id, count]
            }
            Packet::GameCreated(id) => {
                packet![payload; id]
            }
        };
        Message::Binary(payload)
    }
}
