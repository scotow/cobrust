use std::sync::Arc;
use crate::game::Game;
use warp::filters::ws::Message;
use crate::packet;
use crate::misc::ToData;

pub enum Packet<'a> {
    AddGames(Vec<(&'a u16, &'a Arc<Game>)>),
    RemoveGame(u16),
    PlayerCount(u16, u8),
    GameCreated(u16),
}

impl<'a> Packet<'a> {
    pub async fn message(self) -> Message {
        use Packet::*;
        let payload = match self {
            AddGames(games) => {
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
            },
            RemoveGame(id) => {
                packet![1u8, id]
            },
            PlayerCount(id, count) => {
                packet![2u8, id, count]
            },
            GameCreated(id) => {
                packet![3u8, id]
            },
        };
        Message::binary(payload)
    }
}