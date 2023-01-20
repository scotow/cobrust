use axum::extract::ws::Message;
use tokio::sync::MutexGuard;

use crate::{
    game::{
        coordinate::Coord,
        perk::Perk,
        player::{BodyId, Color, Player, PlayerId},
        size::Size,
    },
    misc::PacketSerialize,
    packet,
};

pub enum Packet<'a> {
    Info(Size, &'a str, PlayerId),
    Snakes(Vec<(PlayerId, MutexGuard<'a, Player>)>),
    Perks(Vec<(Coord, Perk)>),
    PlayerJoined(PlayerId, BodyId, Coord, Color),
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
                    packet![packet; id, player.color, player.bodies_len() as u8];
                    for body in player.bodies_iter() {
                        packet![packet; body.id, body.cells.len() as u16];
                        for cell in &body.cells {
                            packet![packet; cell.coord];
                        }
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
            Packet::PlayerJoined(player_id, body_id, head, color) => {
                packet![3u8, player_id, body_id, color, head]
            }
            Packet::PlayerLeft(player_id) => {
                packet![4u8, player_id]
            }
            Packet::ColorChange(player_id, color) => {
                packet![5u8, player_id, color]
            }
            Packet::SnakeChanges(changes) => {
                let mut packet = packet![cap changes.len() * 4; 6u8];
                for change in changes {
                    match change {
                        SnakeChange::RemoveTail(player_id, body_id) => {
                            packet![packet; 0u8, player_id, body_id]
                        }
                        SnakeChange::AddCell(player_id, body_id, coord) => {
                            packet![packet; 1u8, player_id, body_id, coord]
                        }
                        SnakeChange::AddBody(player_id, body_id, coord) => {
                            packet![packet; 2u8, player_id, body_id, coord]
                        }
                        SnakeChange::RemoveBody(player_id, body_id) => {
                            packet![packet; 3u8, player_id, body_id]
                        }
                        SnakeChange::Reverse(player_id) => packet![packet; 4u8, player_id],
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
    RemoveTail(PlayerId, BodyId),
    AddCell(PlayerId, BodyId, Coord),
    AddBody(PlayerId, BodyId, Coord),
    RemoveBody(PlayerId, BodyId),
    Reverse(PlayerId),
}
