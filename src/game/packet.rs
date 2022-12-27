use std::sync::Arc;

use tokio::sync::MutexGuard;
use warp::ws::Message;

use crate::{
    game::{
        coordinate::Coord,
        perk::Perk,
        player::{Player, PlayerId},
        size::Size,
    },
    misc::ToData,
    packet,
};

pub enum Packet<'a> {
    Info(Size, &'a str, PlayerId),
    Snakes(Vec<(PlayerId, MutexGuard<'a, Player>)>),
    Perks(Vec<(Coord, Arc<Box<dyn Perk + Sync + Send>>)>),
    PlayerJoined(PlayerId, Coord, (u16, u16)),
    PlayerLeft(PlayerId),
    SnakeChanges(Vec<SnakeChange>),
}

impl<'a> Packet<'a> {
    pub fn message(self) -> Message {
        let payload = match self {
            Packet::Info(size, name, self_id) => {
                packet![
                    0u8,
                    size.width as u16,
                    size.height as u16,
                    name.as_bytes().len() as u8,
                    name.as_bytes(),
                    self_id
                ]
            }
            Packet::Snakes(players) => {
                let mut packet = Vec::with_capacity(128);
                packet![packet; 1u8];
                for (id, player) in players {
                    packet![packet; id, player.color.0, player.color.1, player.body.len() as u16];
                    for coord in &player.body {
                        packet![packet; coord.x as u16, coord.y as u16];
                    }
                }
                packet
            }
            Packet::Perks(perks) => {
                let mut packet = Vec::with_capacity(perks.len() * 4);
                packet![packet; 2u8];
                for (coord, perk) in perks {
                    packet![packet; coord.x as u16, coord.y as u16, perk];
                }
                packet
            }
            Packet::PlayerJoined(id, head, color) => {
                packet![3u8, id, color.0, color.1, head.x as u16, head.y as u16]
            }
            Packet::PlayerLeft(id) => {
                packet![4u8, id]
            }
            Packet::SnakeChanges(changes) => {
                let mut packet = packet![5u8];
                for change in changes {
                    match change {
                        SnakeChange::Remove(id) => packet![packet; 0u8, id],
                        SnakeChange::Add(id, coord) => {
                            packet![packet; 1u8, id, coord.x as u16, coord.y as u16]
                        }
                        SnakeChange::Die(id, coord) => {
                            packet![packet; 2u8, id, coord.x as u16, coord.y as u16]
                        }
                        SnakeChange::Reverse(id) => packet![packet; 3u8, id],
                    }
                }
                packet
            }
        };
        Message::binary(payload)
    }
}

pub enum SnakeChange {
    Remove(PlayerId),
    Add(PlayerId, Coord),
    Die(PlayerId, Coord),
    Reverse(PlayerId),
}
