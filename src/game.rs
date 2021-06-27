use crate::player::Player;
use tokio::time::sleep;
use std::time::Duration;
use futures::future::join_all;
use std::fmt::Write;
use warp::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::task;
use futures::stream::SplitStream;
use crate::coordinate::Coord;
use std::collections::HashMap;
use crate::perk::{Food, Perk};
use crate::cell::Cell;
use crate::size::Size;
use crate::packet::{Packet, SnakeChange};

pub struct Game {
    size: Size,
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    grid: Vec<Vec<Cell>>,
    players: HashMap<u16, Arc<Mutex<Player>>>,
    perks: HashMap<Coord, Arc<Box<dyn Perk + Send + Sync>>>,
}

impl Game {
    pub fn new(size: Size) -> Self {
        Self {
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

    pub async fn add_player(&self, socket: WebSocket) {
        let mut inner = self.inner.lock().await;

        let id = rand::random();
        let head = self.safe_place(&inner.grid);
        let (mut player, rx) = Player::new(socket, head);
        let _ = player.sink.send(Packet::GridSize(self.size).message().await).await;

        let player = Arc::new(Mutex::new(player));
        self.player_loop(id, Arc::clone(&player), rx);

        inner.players.insert(id, Arc::clone(&player));
        inner.grid[head.y][head.x] = Cell::Occupied;
        Game::broadcast_message(&inner, Packet::PlayerJoined(id, head).message().await).await;

        let snakes_message = Packet::Snakes(&inner.players).message().await;
        let mut player = player.lock().await;
        let _ = player.sink.send(snakes_message).await;
        for &coord in inner.perks.keys() {
            let _ = player.sink.send(Packet::Perk(coord).message().await).await;
        }
    }

    fn player_loop(&self, id: u16, player: Arc<Mutex<Player>>, mut rx: SplitStream<WebSocket>) {
        let inner = Arc::clone(&self.inner);
        task::spawn(async move {
            while let Some(Ok(message)) = rx.next().await {
                if message.is_close() {
                    break;
                }
                player.lock().await.process(message).await;
            }
            // Player left the game from here.

            let mut inner = inner.lock().await;
            inner.players.remove(&id);
            player.lock().await.body.iter()
                .for_each(|&c| inner.grid[c.y][c.x] = Cell::Empty);
            Game::broadcast_message(&inner, Packet::PlayerLeft(id).message().await).await;
        });
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
                    p.lock().await.sink.send(message).await.unwrap();
                }
            })
        ).await;
    }

    #[allow(dead_code)]
    async fn broadcast_grid(&self, inner: &Inner) {
        Game::broadcast_message(
            inner,
            Message::binary([vec![1], self.bytes_grid(&inner.grid)].concat()),
        ).await;
    }

    #[allow(dead_code)]
    fn ascii_grid(&self, grid: &[Vec<Cell>]) -> String {
        let mut ascii = String::with_capacity(grid.len() * (grid[0].len() + 1));
        for row in grid {
            for cell in row {
                ascii.write_char(cell.as_char()).unwrap();
            }
            ascii.write_char('\n').unwrap();
        }
        ascii
    }

    fn bytes_grid(&self, grid: &[Vec<Cell>]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.size.width * self.size.height);
        for row in grid {
            for cell in row {
                bytes.push(cell.as_u8());
            }
        }
        bytes
    }
}