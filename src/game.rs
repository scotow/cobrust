use crate::player::Player;
use tokio::time::sleep;
use std::time::Duration;
use futures::future::join_all;
use warp::ws::{Message, WebSocket};
use futures::StreamExt;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::coordinate::Coord;
use std::collections::HashMap;
use crate::perk::{Food, Perk};
use crate::cell::Cell;
use crate::size::Size;
use crate::packet::{Packet, SnakeChange};
use futures::stream::SplitStream;

pub struct Game {
    pub name: String,
    pub size: Size,
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    grid: Vec<Vec<Cell>>,
    players: HashMap<u16, Arc<Mutex<Player>>>,
    perks: HashMap<Coord, Arc<Box<dyn Perk + Send + Sync>>>,
}

impl Game {
    pub fn new(name: String, size: Size) -> Self {
        Self {
            name,
            size,
            inner: Arc::new(Mutex::new(Inner {
                grid: vec![vec![Cell::Empty; size.width]; size.height],
                players: HashMap::new(),
                perks: HashMap::new(),
            })),
        }
    }

    pub async fn run(&self) {
        let mut inner = self.inner.lock().await;
        for _ in 0..5 {
            self.spawn_food(&mut inner).await;
        }
        drop(inner);

        loop {
            let mut inner = self.inner.lock().await;
            if !inner.players.is_empty() {
                self.walk_snakes(&mut inner).await;
            }
            drop(inner);
            sleep(Duration::from_millis(50)).await;
        }
    }

    pub async fn play(&self, socket: WebSocket) {
        let mut inner = self.inner.lock().await;

        let head = self.safe_place(&inner.grid);
        let (tx, rx) = socket.split();
        let mut player = Player::new(head, tx);
        let _ = player.send(Packet::GridSize(self.size).message().await).await;

        let id = rand::random();
        let player = Arc::new(Mutex::new(player));
        inner.players.insert(id, Arc::clone(&player));
        inner.grid[head.y][head.x] = Cell::Occupied;
        Game::broadcast_message(&inner, Packet::PlayerJoined(id, head).message().await).await;

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
                0 => break,
                1 => player.lock().await.process(&data[1..]).await,
                _ => (),
            }
        };
        // Player left the game from here.
    }

    fn safe_place(&self, grid: &[Vec<Cell>]) -> Coord {
        loop {
            let coord = Coord::random(self.size);
            if matches!(grid[coord.y][coord.x], Cell::Empty) {
                return coord;
            }
        }
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

                    let head = self.safe_place(&inner.grid);
                    player.respawn(head).await;
                    inner.grid[head.y][head.x] = Cell::Occupied;
                    payload.push(SnakeChange::Die(id, head));
                },
                Cell::Perk(perk) => {
                    perk.consume(&mut *player.lock().await);
                    inner.perks.remove(&new);

                    *target = Cell::Occupied;
                    payload.push(SnakeChange::Add(id, new));

                    self.spawn_food(inner).await;
                },
            }
        }
        Game::broadcast_message(inner, Packet::SnakeChanges(payload).message().await).await;
    }

    async fn spawn_food(&self, inner: &mut Inner) {
        let coord = self.safe_place(&inner.grid);
        let food: Arc<Box<dyn Perk + Send + Sync>> = Arc::new(Box::new(Food));
        inner.grid[coord.y][coord.x] = Cell::Perk(Arc::clone(&food));
        inner.perks.insert(coord, food);
        Game::broadcast_message(inner, Packet::Perk(coord).message().await).await;
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