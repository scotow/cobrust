use crate::player::Player;
use tokio::time::sleep;
use std::time::Duration;
use futures::future::join_all;
use std::fmt::{Debug, Write};
use warp::ws::{Message, WebSocket};
use futures::{SinkExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::task;
use futures::stream::SplitStream;

#[derive(Copy, Clone, Debug)]
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
    grid: Arc<Mutex<Vec<Vec<Cell>>>>,
    players: Arc<Mutex<Vec<Arc<Player>>>>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            grid: Arc::new(Mutex::new(vec![vec![Cell::Empty; 16]; 16])),
            players: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn run(&self) {
        loop {
            let players = self.players.lock().await;
            self.walk_snakes(&*players).await;
            self.broadcast_grid(&*players).await;
            drop(players);

            sleep(Duration::from_millis(50)).await;
        }
    }

    async fn walk_snakes(&self, players: &[Arc<Player>]) {
        let changes = join_all(players.iter().map(|p| p.walk())).await;
        let mut grid = self.grid.lock().await;
        changes.into_iter()
            .filter_map(|c| c)
            .for_each(|(r, n)| {
                if let Some(removed) = r {
                    grid[removed.y][removed.x] = Cell::Empty;
                }
                grid[n.y][n.x] = Cell::Occupied;
            });
    }

    async fn broadcast_grid(&self, players: &[Arc<Player>]) {
        let message = Message::text(self.ascii_grid().await);
        join_all(players.iter()
            .map(|p| {
                let message = message.clone();
                async move {
                    let _ = p.sink.lock().await.send(message).await;
                }
            })
        ).await;
    }

    pub async fn add_player(&self, socket: WebSocket) {
        let (player, rx, head) = Player::new(socket);
        let player = Arc::new(player);
        self.player_loop(Arc::clone(&player), rx);

        self.grid.lock().await[head.y][head.x] = Cell::Occupied;
        self.players.lock().await.push(player);
    }

    fn player_loop(&self, player: Arc<Player>, rx: SplitStream<WebSocket>) {
        let players = Arc::clone(&self.players);
        let grid = Arc::clone(&self.grid);
        task::spawn(async move {
            player.listen(rx).await;

            let mut players = players.lock().await;
            if let Some(index) = players.iter().position(|p| p.id == player.id) {
                players.remove(index);
            }
            drop(players);

            let mut grid = grid.lock().await;
            player.body.lock().await.iter()
                .for_each(|&c| grid[c.y][c.x] = Cell::Empty);
        });
    }

    async fn ascii_grid(&self) -> String {
        let grid = self.grid.lock().await;
        let mut ascii = String::with_capacity(grid.len() * (grid[0].len() + 1));
        for row in &*grid {
            for &cell in row {
                ascii.write_char(cell.into()).unwrap();
            }
            ascii.write_char('\n').unwrap();
        }
        ascii
    }
}