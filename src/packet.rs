use crate::size::Size;
use crate::player::Player;
use warp::ws::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::coordinate::Coord;
use crate::game::Game;

macro_rules! packet {
    [$($elem:expr),*] => {
        {
            let mut packet = Vec::with_capacity(32);
            $(
                $elem.push(&mut packet);
            )*
            packet
        }
    };
    [$vec:expr; $($elem:expr),*] => {
        {
            $(
                $elem.push(&mut $vec);
            )*
        }
    };
}

pub enum Packet<'a> {
    Games(Vec<(&'a u16, &'a Arc<Game>)>),
    GamePlayerCount(u16, u8),
    GameCreated(u16),
    GameInfo(Size, &'a str),
    Snakes(&'a HashMap<u16, Arc<Mutex<Player>>>),
    Perk(Coord),
    PlayerJoined(u16, Coord, (u16, u16)),
    PlayerLeft(u16),
    SnakeChanges(Vec<SnakeChange>),
}

impl<'a> Packet<'a> {
    pub async fn message(self) -> Message {
        use Packet::*;
        let payload = match self {
            Games(games) => {
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
            GamePlayerCount(id, count) => {
                packet![1u8, id, count]
            },
            GameCreated(id) => {
                packet![2u8, id]
            }
            GameInfo(size, name) => {
                packet![0u8, size.width as u16, size.height as u16, name.as_bytes().len() as u8, name.as_bytes()]
            }
            Snakes(players) => {
                let mut packet = Vec::with_capacity(128);
                packet.push(1);
                for (&id, player) in players {
                    let player = player.lock().await;
                    packet![packet; id, player.color.0, player.color.1, player.body.len() as u16];
                    for coord in &player.body {
                        packet![packet; coord.x as u16, coord.y as u16];
                    }
                }
                packet
            },
            Perk(coord) => {
                packet![2u8, coord.x as u16, coord.y as u16]
            }
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

impl SnakeChange {
    #[allow(dead_code)]
    fn id(&self) -> u8 {
        use SnakeChange::*;
        match self {
            Remove(_) => 0,
            Add(_, _) => 1,
            Die(_, _) => 2,
        }
    }
}

trait ToData {
    fn push(&self, out: &mut Vec<u8>);
}
impl ToData for u8 {
    fn push(&self, out: &mut Vec<u8>) { out.push(*self) }
}
impl ToData for u16 {
    fn push(&self, out: &mut Vec<u8>) { out.extend_from_slice(&self.to_be_bytes()) }
}
impl ToData for [u8] {
    fn push(&self, out: &mut Vec<u8>) { out.extend_from_slice(self) }
}