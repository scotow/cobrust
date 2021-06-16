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
use crate::perk::Food;
use crate::cell::Cell;
use crate::size::Size;

pub struct Game {
    size: Size,
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    grid: Vec<Vec<Cell>>,
    players: HashMap<usize, Arc<Mutex<Player>>>,
}

impl Game {
    pub fn new() -> Self {
        let size = Size { width: 60, height: 40 };
        Self {
            size,
            inner: Arc::new(Mutex::new(Inner {
                grid: vec![vec![Cell::Empty; size.width]; size.height],
                players: HashMap::new(),
            })),
        }
    }

    pub async fn run(&self) {
        {
            let mut inner = self.inner.lock().await;
            self.spawn_food(&mut inner.grid);
        }

        loop {
            {
                let mut inner = self.inner.lock().await;
                self.walk_snakes(&mut inner).await;
                self.broadcast_grid(&inner).await;
            }
            sleep(Duration::from_millis(50)).await;
        }
    }

    pub async fn add_player(&self, socket: WebSocket) {
        let mut inner = self.inner.lock().await;

        let id = rand::random();
        let head = self.safe_place(&inner.grid);
        let (mut player, rx) = Player::new(socket, head);
        let _ = player.sink.send(Message::binary(vec![0, self.size.width as u8, self.size.height as u8])).await;

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
        loop {
            let coord = Coord::random(self.size);
            if matches!(grid[coord.y][coord.x], Cell::Empty) {
                return coord;
            }
        }
    }

    async fn walk_snakes(&self, inner: &mut Inner) {
        let changes = join_all(
            inner.players.values().map(|p| {
                async move {
                    p.lock().await
                        .walk(self.size).await
                        .map(|cs| (Arc::clone(p), cs))
                }
            })).await;

        for (player, (removed, new)) in changes.into_iter()
            .filter_map(|e| e) {
            if let Some(removed) = removed {
                inner.grid[removed.y][removed.x] = Cell::Empty;
            }

            let target = &mut inner.grid[new.y][new.x];
            match target {
                Cell::Empty => {
                    *target = Cell::Occupied;
                },
                Cell::Occupied => {
                    let mut player = player.lock().await;
                    player.body.iter().skip(1).for_each(|&c| inner.grid[c.y][c.x] = Cell::Empty);

                    let head = self.safe_place(&inner.grid);
                    player.respawn(head).await;
                    inner.grid[head.y][head.x] = Cell::Occupied;
                },
                Cell::Perk(perk) => {
                    let mut player = player.lock().await;
                    perk.consume(&mut player);
                    *target = Cell::Occupied;
                    self.spawn_food(&mut inner.grid);
                },
            }
        }
    }

    fn spawn_food(&self, grid: &mut [Vec<Cell>]) {
        let food = self.safe_place(grid);
        grid[food.y][food.x] = Cell::Perk(Box::new(Food));
    }

    async fn broadcast_grid(&self, inner: &Inner) {
        let message = Message::binary([vec![1], self.bytes_grid(&inner.grid)].concat());
        join_all(inner.players.values()
            .map(|p| {
                let message = message.clone();
                async move {
                    let _ = p.lock().await.sink.send(message).await;
                }
            })
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