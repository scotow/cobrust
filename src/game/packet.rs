use axum::extract::ws::Message;
use tokio::sync::MutexGuard;

use crate::{
    game::{
        coordinate::Coord,
        perk::Perk,
        player::{Color, Player, PlayerId},
        size::Size,
    },
    misc::PacketSerialize,
    packet,
};

pub enum Packet<'a> {
    Info(Size, &'a str, PlayerId),
    Snakes(Vec<(PlayerId, MutexGuard<'a, Player>)>),
    Perks(Vec<(Coord, Perk)>),
    PlayerJoined(PlayerId, Coord, Color),
    PlayerLeft(PlayerId),
    ColorChange(PlayerId, Color),
    SnakeChanges(Vec<SnakeChange>),
}

impl<'a> Packet<'a> {
    pub fn message(self) -> Message {
        let payload = match self {
            Packet::Info(size, name, self_id) => {
                packet![
                    0u8,
                    size,
                    name.as_bytes().len() as u8,
                    name.as_bytes(),
                    self_id
                ]
            }
            Packet::Snakes(players) => {
                let mut packet = packet![cap players.len() * 64; 1u8];
                for (id, player) in players {
                    packet![packet; id, player.color, player.body.len() as u16];
                    for cell in &player.body {
                        packet![packet; cell.coord];
                    }
                }
                packet
            }
            Packet::Perks(perks) => {
                let mut packet = packet![cap perks.len() * 4; 2u8];
                for (coord, perk) in perks {
                    packet![packet; coord, perk];
                }
                packet
            }
            Packet::PlayerJoined(id, head, color) => {
                packet![3u8, id, color, head]
            }
            Packet::PlayerLeft(id) => {
                packet![4u8, id]
            }
            Packet::ColorChange(id, color) => {
                packet![5u8, id, color]
            }
            Packet::SnakeChanges(changes) => {
                let mut packet = packet![cap changes.len() * 4; 6u8];
                for change in changes {
                    match change {
                        SnakeChange::RemoveTail(id) => packet![packet; 0u8, id],
                        SnakeChange::AddCell(id, coord) => {
                            packet![packet; 1u8, id, coord]
                        }
                        SnakeChange::Die(id, coord) => {
                            packet![packet; 2u8, id, coord]
                        }
                        SnakeChange::Reverse(id) => packet![packet; 3u8, id],
                    }
                }
                packet
            }
        };
        Message::Binary(payload)
    }
}

#[derive(Debug)]
pub enum SnakeChange {
    RemoveTail(PlayerId),
    AddCell(PlayerId, Coord),
    Die(PlayerId, Coord),
    Reverse(PlayerId),
}
