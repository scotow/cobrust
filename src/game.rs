use crate::player::Player;
use tokio::time::sleep;
use std::time::Duration;
use futures::future::join_all;
use std::fmt::{Debug, Write};
use warp::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::task;
use futures::stream::SplitStream;
use crate::coord::Coord;
use rand::Rng;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cell {
    Empty,
    Perk,
    Occupied,
}

impl Into<char> for Cell {
    fn into(self) -> char {
        use Cell::*;
        match self {
            Empty => '.',
            Perk => unreachable!(),
            Occupied => '#',
        }
    }
}

pub struct Game {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    grid: Vec<Vec<Cell>>,
    players: HashMap<usize, Arc<Mutex<Player>>>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                grid: vec![vec![Cell::Empty; 16]; 16],
                players: HashMap::new(),
            }))
        }
    }

    pub async fn run(&self) {
        loop {
            let mut inner = self.inner.lock().await;
            self.walk_snakes(&mut inner).await;
            self.broadcast_grid(&inner).await;
            drop(inner);

            sleep(Duration::from_millis(50)).await;
        }
    }

    pub async fn add_player(&self, socket: WebSocket) {
        let mut inner = self.inner.lock().await;
        let head = self.safe_place(&inner.grid);

        let id = rand::random();
        let (player, rx) = Player::new(socket, head);
        let player = Arc::new(Mutex::new(player));
        self.player_loop(id, Arc::clone(&player), rx);

        inner.grid[head.y][head.x] = Cell::Occupied;
        inner.players.insert(id, player);
    }

    fn player_loop(&self, id: usize, player: Arc<Mutex<Player>>, mut rx: SplitStream<WebSocket>) {
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
        });
    }

    fn safe_place(&self, grid: &[Vec<Cell>]) -> Coord {
        let mut rng = rand::thread_rng();
        loop {
            let tmp = Coord {
                x: rng.gen_range(0..16),
                y: rng.gen_range(0..16),
            };
            if grid[tmp.y][tmp.x] == Cell::Empty {
                return tmp;
            }
        }
    }

    async fn walk_snakes(&self, inner: &mut Inner) {
        let changes = join_all(
            inner.players.values().map(|p| {
                async move {
                    p.lock().await
                        .walk().await
                        .map(|cs| (Arc::clone(p), cs))
                }
            })).await;

        for (player, (removed, new)) in changes.into_iter()
            .filter_map(|e| e) {
            if let Some(removed) = removed {
                inner.grid[removed.y][removed.x] = Cell::Empty;
            }
            if inner.grid[new.y][new.x] == Cell::Occupied {
                let mut player = player.lock().await;
                player.body.iter().skip(1).for_each(|&c| inner.grid[c.y][c.x] = Cell::Empty);

                let head = self.safe_place(&inner.grid);
                player.respawn(head).await;
                inner.grid[head.y][head.x] = Cell::Occupied;
            } else {
                inner.grid[new.y][new.x] = Cell::Occupied;
            }
        }
    }

    async fn broadcast_grid(&self, inner: &Inner) {
        let message = Message::text(self.ascii_grid(&inner.grid).await);
        join_all(inner.players.values()
            .map(|p| {
                let message = message.clone();
                async move {
                    let _ = p.lock().await.sink.send(message).await;
                }
            })
        ).await;
    }

    async fn ascii_grid(&self, grid: &[Vec<Cell>]) -> String {
        let mut ascii = String::with_capacity(grid.len() * (grid[0].len() + 1));
        for row in grid {
            for &cell in row {
                ascii.write_char(cell.into()).unwrap();
            }
            ascii.write_char('\n').unwrap();
        }
        ascii
    }
}