pub mod cell;
pub mod config;
pub mod coordinate;
pub mod direction;
pub mod packet;
pub mod perk;
pub mod player;
pub mod size;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future::join_all;
use futures::stream::SplitStream;
use futures::StreamExt;
use tokio::sync::Mutex;
use tokio::time::sleep;
use warp::ws::{Message, WebSocket};

use cell::Cell;
use coordinate::Coord;
use player::Player;
use size::Size;
use packet::Packet;
use crate::game::packet::SnakeChange;
use crate::game::perk::{Perk, Generator};
use crate::game::config::Config;

pub struct Game {
    pub name: String,
    pub size: Size,
    pub speed: u8,
    pub food_strength: u16,
    inner: Arc<Mutex<Inner>>,
}

impl Game {
    pub fn new(config: Config) -> Self {
        let mut inner = Inner {
            grid: vec![vec![Cell::Empty; config.size.width as usize]; config.size.height as usize],
            players: HashMap::new(),
            perks: HashMap::new(),
            perk_generator: Generator::new(config.food_strength, config.reserved_food),
            last_leave: Instant::now(),
        };
        for _ in 0..(config.foods as usize) {
            inner.add_perk(config.size, Arc::new(Box::new(inner.perk_generator.fresh_food())));
        }

        Self {
            name: config.name,
            size: config.size,
            speed: config.speed,
            food_strength: config.food_strength,
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub async fn run(&self) {
        loop {
            let mut inner = self.inner.lock().await;
            if inner.players.is_empty() {
                if inner.last_leave.elapsed() > Duration::from_secs(60) {
                    break;
                }
                drop(inner);
                sleep(Duration::from_millis(500)).await;
            } else {
                self.walk_snakes(&mut inner).await;
                drop(inner);
                sleep(Duration::from_millis(1000 / self.speed as u64)).await;
            }
        }
    }

    pub async fn play(&self, socket: WebSocket) {
        let mut inner = self.inner.lock().await;

        let head = inner.safe_place(self.size);
        let (tx, rx) = socket.split();
        let id = rand::random();
        let mut player = Player::new(head, tx);
        let color = player.color;
        let _ = player.send(Packet::Info(self.size, &self.name, id).message().await).await;

        let player = Arc::new(Mutex::new(player));
        inner.players.insert(id, Arc::clone(&player));
        inner.grid[head.y][head.x] = Cell::Occupied;
        inner.broadcast_message(Packet::PlayerJoined(id, head, color).message().await).await;

        {
            let snakes_message = Packet::Snakes(&inner.players).message().await;
            let mut player = player.lock().await;
            let _ = player.send(snakes_message).await;
            for (coord, perk) in &inner.perks {
                let _ = player.send(Packet::Perk(*coord, Arc::clone(perk)).message().await).await;
            }
        }
        drop(inner);

        self.player_loop(player, rx).await;
        let mut inner = self.inner.lock().await;
        let player = inner.players.remove(&id).unwrap();
        player.lock().await.body.iter()
            .for_each(|&c| inner.grid[c.y][c.x] = Cell::Empty);
        inner.last_leave = Instant::now();
        inner.broadcast_message(Packet::PlayerLeft(id).message().await).await;
    }

    async fn player_loop(&self, player: Arc<Mutex<Player>>, mut rx: SplitStream<WebSocket>) {
        loop {
            let message = match rx.next().await {
                Some(Ok(message)) => message,
                _ => break,
            };
            if message.is_close() {
                break;
            }

            let data = message.as_bytes();
            match data[0] {
                0 => player.lock().await.process(&data[1..]).await,
                _ => (),
            }
        };
        // Player left the game from here.
    }

    async fn walk_snakes(&self, inner: &mut Inner) {
        let changes = join_all(
            inner.players.iter().map(|(&id, p)| {
                async move {
                    p.lock().await
                        .walk(self.size).await
                        .map(|cs| (id, Arc::clone(p), cs))
                }
            })).await;

        let mut payload = Vec::with_capacity(changes.len() * 2);
        for (id, player, (removed, new)) in changes.into_iter()
            .filter_map(|e| e) {
            if let Some(removed) = removed {
                inner.grid[removed.y][removed.x] = Cell::Empty;
                payload.push(SnakeChange::Remove(id));
            }

            let target = &mut inner.grid[new.y][new.x];
            match target {
                Cell::Empty => {
                    *target = Cell::Occupied;
                    payload.push(SnakeChange::Add(id, new));
                },
                Cell::Occupied => {
                    let mut player = player.lock().await;
                    player.body.iter().skip(1).for_each(|&c| inner.grid[c.y][c.x] = Cell::Empty);

                    let head = inner.safe_place(self.size);
                    player.respawn(head).await;
                    inner.grid[head.y][head.x] = Cell::Occupied;
                    payload.push(SnakeChange::Die(id, head));
                },
                Cell::Perk(perk) => {
                    perk.consume((id, &mut *player.lock().await));
                    inner.perks.remove(&new);

                    let need_food = perk.make_spawn_food();
                    *target = Cell::Occupied;
                    payload.push(SnakeChange::Add(id, new));

                    if need_food {
                        for perk in inner.perk_generator.next(id) {
                            let perk = Arc::new(perk);
                            let coord = inner.add_perk(self.size, Arc::clone(&perk));
                            inner.broadcast_message(Packet::Perk(coord, perk).message().await).await;
                        }
                    }
                },
            }
        }
        inner.broadcast_message(Packet::SnakeChanges(payload).message().await).await;
    }

    pub async fn player_count(&self) -> usize {
        self.inner.lock().await.players.len()
    }
}

struct Inner {
    grid: Vec<Vec<Cell>>,
    players: HashMap<u16, Arc<Mutex<Player>>>,
    perks: HashMap<Coord, Arc<Box<dyn Perk + Send + Sync>>>,
    perk_generator: Generator,
    last_leave: Instant,
}

impl Inner {
    fn safe_place(&self, size: Size) -> Coord {
        loop {
            let coord = Coord::random(size);
            if matches!(self.grid[coord.y][coord.x], Cell::Empty) {
                return coord;
            }
        }
    }

    fn add_perk(&mut self, size: Size, perk: Arc<Box<dyn Perk + Send + Sync>>) -> Coord {
        let coord = self.safe_place(size);
        self.grid[coord.y][coord.x] = Cell::Perk(Arc::clone(&perk));
        self.perks.insert(coord, Arc::clone(&perk));
        coord
    }

    async fn broadcast_message(&self, message: Message) {
        if self.players.is_empty() {
            return;
        }
        join_all(self.players.values()
            .map(|p| {
                let message = message.clone();
                async move {
                    p.lock().await.send(message).await;
                }
            })
        ).await;
    }
}
