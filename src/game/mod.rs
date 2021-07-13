pub mod cell;
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
use crate::game::perk::{Perk, Food};

pub struct Game {
    pub name: String,
    pub size: Size,
    pub speed: u8,
    pub food_strength: u16,
    inner: Arc<Mutex<Inner>>,
}

pub struct Config {
    pub name: String,
    pub size: Size,
    pub speed: u8,
    pub foods: u16,
    pub food_strength: u16,
}

impl Config {
    pub fn is_valid(&self) -> bool {
        (1..=32).contains(&self.name.len()) &&
            (16..=255).contains(&self.size.width) &&
            (16..=255).contains(&self.size.height) &&
            (1..=50).contains(&self.speed) &&
            (1..=32).contains(&self.foods) &&
            (0..=1024).contains(&self.food_strength)
    }
}

impl Game {
    pub fn new(config: Config) -> Self {
        let mut inner = Inner {
            grid: vec![vec![Cell::Empty; config.size.width as usize]; config.size.height as usize],
            players: HashMap::new(),
            perks: HashMap::new(),
            last_leave: Instant::now(),
        };
        for _ in 0..(config.foods as usize) {
            inner.spawn_food(config.size, config.food_strength);
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
            } else {
                self.walk_snakes(&mut inner).await;
            }
            drop(inner);
            sleep(Duration::from_millis(1000 / self.speed as u64)).await;
        }
    }

    pub async fn play(&self, socket: WebSocket) {
        let mut inner = self.inner.lock().await;

        let head = inner.safe_place(self.size);
        let (tx, rx) = socket.split();
        let mut player = Player::new(head, tx);
        let color = player.color;
        let _ = player.send(Packet::GameInfo(self.size, &self.name).message().await).await;

        let id = rand::random();
        let player = Arc::new(Mutex::new(player));
        inner.players.insert(id, Arc::clone(&player));
        inner.grid[head.y][head.x] = Cell::Occupied;
        Game::broadcast_message(&inner, Packet::PlayerJoined(id, head, color).message().await).await;

        {
            let snakes_message = Packet::Snakes(&inner.players).message().await;
            let mut player = player.lock().await;
            let _ = player.send(snakes_message).await;
            for &coord in inner.perks.keys() {
                let _ = player.send(Packet::Perk(coord).message().await).await;
            }
        }
        drop(inner);

        self.player_loop(player, rx).await;
        let mut inner = self.inner.lock().await;
        let player = inner.players.remove(&id).unwrap();
        player.lock().await.body.iter()
            .for_each(|&c| inner.grid[c.y][c.x] = Cell::Empty);
        inner.last_leave = Instant::now();
        Game::broadcast_message(&inner, Packet::PlayerLeft(id).message().await).await;
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
                    perk.consume(&mut *player.lock().await);
                    inner.perks.remove(&new);

                    *target = Cell::Occupied;
                    payload.push(SnakeChange::Add(id, new));

                    let new_food = inner.spawn_food(self.size, self.food_strength);
                    Game::broadcast_message(inner, Packet::Perk(new_food).message().await).await;
                },
            }
        }
        Game::broadcast_message(inner, Packet::SnakeChanges(payload).message().await).await;
    }

    async fn broadcast_message(inner: &Inner, message: Message) {
        if inner.players.is_empty() {
            return;
        }
        join_all(inner.players.values()
            .map(|p| {
                let message = message.clone();
                async move {
                    p.lock().await.send(message).await;
                }
            })
        ).await;
    }

    pub async fn player_count(&self) -> usize {
        self.inner.lock().await.players.len()
    }
}

struct Inner {
    grid: Vec<Vec<Cell>>,
    players: HashMap<u16, Arc<Mutex<Player>>>,
    perks: HashMap<Coord, Arc<Box<dyn Perk + Send + Sync>>>,
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

    fn spawn_food(&mut self, size: Size, strength: u16) -> Coord {
        let coord = self.safe_place(size);
        let food: Arc<Box<dyn Perk + Send + Sync>> = Arc::new(Box::new(Food(strength)));
        self.grid[coord.y][coord.x] = Cell::Perk(Arc::clone(&food));
        self.perks.insert(coord, food);
        coord
    }
}
