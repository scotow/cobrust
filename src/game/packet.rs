use axum::extract::ws::Message;
use enum_index::EnumIndex;
use enum_index_derive::EnumIndex;
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

#[derive(EnumIndex)]
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
        let mut payload = packet![cap 256; self.enum_index() as u8];
        match self {
            Packet::Info(size, name, self_id) => {
                packet![
                    payload;
                    size,
                    name.as_bytes().len() as u8,
                    name.as_bytes(),
                    self_id
                ]
            }
            Packet::Snakes(players) => {
                for (id, player) in players {
                    packet![payload; id, player.color, player.bodies_len() as u8];
                    for body in player.bodies_iter() {
                        packet![payload; body.id, body.cells.len() as u16];
                        for cell in &body.cells {
                            packet![payload; cell.coord];
                        }
                    }
                }
            }
            Packet::Perks(perks) => {
                for (coord, perk) in perks {
                    packet![payload; coord, perk];
                }
            }
            Packet::PlayerJoined(player_id, body_id, head, color) => {
                packet![payload; player_id, body_id, color, head]
            }
            Packet::PlayerLeft(player_id) => {
                packet![payload; player_id]
            }
            Packet::ColorChange(player_id, color) => {
                packet![payload; player_id, color]
            }
            Packet::SnakeChanges(changes) => {
                for change in changes {
                    packet![payload; change.enum_index() as u8];
                    match change {
                        SnakeChange::RemoveTail(player_id, body_id) => {
                            packet![payload; player_id, body_id]
                        }
                        SnakeChange::AddCell(player_id, body_id, coord) => {
                            packet![payload; player_id, body_id, coord]
                        }
                        SnakeChange::AddBody(player_id, body_id, coord) => {
                            packet![payload; player_id, body_id, coord]
                        }
                        SnakeChange::RemoveBody(player_id, body_id) => {
                            packet![payload; player_id, body_id]
                        }
                        SnakeChange::Reverse(player_id) => {
                            packet![payload; player_id]
                        }
                    }
                }
            }
        };
        Message::Binary(payload)
    }
}

#[derive(EnumIndex, Debug)]
pub enum SnakeChange {
    RemoveTail(PlayerId, BodyId),
    AddCell(PlayerId, BodyId, Coord),
    AddBody(PlayerId, BodyId, Coord),
    RemoveBody(PlayerId, BodyId),
    Reverse(PlayerId),
}
