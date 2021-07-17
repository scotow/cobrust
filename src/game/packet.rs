use crate::game::size::Size;
use crate::game::player::Player;
use warp::ws::Message;
use std::sync::Arc;
use tokio::sync::MutexGuard;
use crate::game::coordinate::Coord;
use crate::packet;
use crate::misc::ToData;
use crate::game::perk::Perk;

pub enum Packet<'a> {
    Info(Size, &'a str, u16),
    Snakes(Vec<(u16, MutexGuard<'a, Player>)>),
    Perks(Vec<(Coord, Arc<Box<dyn Perk + Sync + Send>>)>),
    PlayerJoined(u16, Coord, (u16, u16)),
    PlayerLeft(u16),
    SnakeChanges(Vec<SnakeChange>),
}

impl<'a> Packet<'a> {
    pub fn message(self) -> Message {
        use Packet::*;
        let payload = match self {
            Info(size, name, self_id) => {
                packet![0u8, size.width as u16, size.height as u16, name.as_bytes().len() as u8, name.as_bytes(), self_id]
            }
            Snakes(players) => {
                let mut packet = Vec::with_capacity(128);
                packet![packet; 1u8];
                for (id, player) in players {
                    packet![packet; id, player.color.0, player.color.1, player.body.len() as u16];
                    for coord in &player.body {
                        packet![packet; coord.x as u16, coord.y as u16];
                    }
                }
                packet
            },
            Perks(perks) => {
                let mut packet = Vec::with_capacity(perks.len() * 4);
                packet![packet; 2u8];
                for (coord, perk) in perks {
                    packet![packet; coord.x as u16, coord.y as u16, perk];
                }
                packet
            },
            PlayerJoined(id, head, color) => {
                packet![3u8, id, color.0, color.1, head.x as u16, head.y as u16]
            },
            PlayerLeft(id) => {
                packet![4u8, id]
            }
            SnakeChanges(changes) => {
                use SnakeChange::*;
                let mut packet = packet![5u8];
                for change in changes {
                    match change {
                        Remove(id) => packet![packet; 0u8, id],
                        Add(id, coord) => packet![packet; 1u8, id, coord.x as u16, coord.y as u16],
                        Die(id, coord) => packet![packet; 2u8, id, coord.x as u16, coord.y as u16],
                    }
                }
                packet
            },
        };
        Message::binary(payload)
    }
}

pub enum SnakeChange {
    Remove(u16),
    Add(u16, Coord),
    Die(u16, Coord),
}