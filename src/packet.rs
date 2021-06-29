use crate::size::Size;
use crate::player::Player;
use warp::ws::Message;
use byteorder::{WriteBytesExt, BE};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::coordinate::Coord;
use crate::game::Game;
use std::io::Write;

pub enum Packet<'a> {
    Games(&'a HashMap<u16, Arc<Game>>),
    GamePlayerCount(u16, u8),
    GridSize(Size),
    Snakes(&'a HashMap<u16, Arc<Mutex<Player>>>),
    Perk(Coord),
    PlayerJoined(u16, Coord),
    PlayerLeft(u16),
    SnakeChanges(Vec<SnakeChange>),
}

impl<'a> Packet<'a> {
    fn id(&self) -> u8 {
        use Packet::*;
        match self {
            Games(_) => 0,
            GamePlayerCount(_, _) => 1,
            GridSize(_) => 0,
            Snakes(_) => 1,
            Perk(_) => 2,
            PlayerJoined(_, _) => 3,
            PlayerLeft(_) => 4,
            SnakeChanges(_) => 5,
        }
    }

    pub async fn message(self) -> Message {
        use Packet::*;
        let mut payload = vec![self.id()];
        match self {
            Games(games) => {
                for (&id, game) in games {
                    payload.write_u16::<BE>(id).unwrap();
                    payload.write_u8(game.name.as_bytes().len() as u8).unwrap();
                    payload.write_all(game.name.as_bytes()).unwrap();
                    payload.write_u16::<BE>(game.size.width as u16).unwrap();
                    payload.write_u16::<BE>(game.size.height as u16).unwrap();
                    payload.write_u8(game.player_count().await as u8).unwrap();
                }
            }
            GamePlayerCount(id, count) => {
                payload.write_u16::<BE>(id).unwrap();
                payload.write_u8(count).unwrap();
            },
            GridSize(size) => {
                payload.write_u16::<BE>(size.width as u16).unwrap();
                payload.write_u16::<BE>(size.height as u16).unwrap();
            }
            Snakes(players) => {
                for (&id, player) in players {
                    payload.write_u16::<BE>(id).unwrap();
                    let body = &player.lock().await.body;
                    payload.write_u16::<BE>(body.len() as u16).unwrap();
                    for &coord in body {
                        payload.write_u16::<BE>(coord.x as u16).unwrap();
                        payload.write_u16::<BE>(coord.y as u16).unwrap();
                    }
                }
            },
            Perk(coord) => {
                payload.write_u16::<BE>(coord.x as u16).unwrap();
                payload.write_u16::<BE>(coord.y as u16).unwrap();
            }
            PlayerJoined(id, head) => {
                payload.write_u16::<BE>(id).unwrap();
                payload.write_u16::<BE>(head.x as u16).unwrap();
                payload.write_u16::<BE>(head.y as u16).unwrap();
            },
            PlayerLeft(id) => {
                payload.write_u16::<BE>(id).unwrap();
            }
            SnakeChanges(changes) => {
                use SnakeChange::*;
                for change in changes {
                    payload.write_u8(change.id()).unwrap();
                    match change {
                        Remove(id) => payload.write_u16::<BE>(id).unwrap(),
                        Add(id, coord) => {
                            payload.write_u16::<BE>(id).unwrap();
                            payload.write_u16::<BE>(coord.x as u16).unwrap();
                            payload.write_u16::<BE>(coord.y as u16).unwrap();
                        },
                        Die(id, coord) => {
                            payload.write_u16::<BE>(id).unwrap();
                            payload.write_u16::<BE>(coord.x as u16).unwrap();
                            payload.write_u16::<BE>(coord.y as u16).unwrap();
                        },
                    }
                }
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
    fn id(&self) -> u8 {
        use SnakeChange::*;
        match self {
            Remove(_) => 0,
            Add(_, _) => 1,
            Die(_, _) => 2,
        }
    }
}