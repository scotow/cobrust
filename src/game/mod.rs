use std::{
    collections::HashMap,
    iter,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::extract::ws::{Message, WebSocket};
use cell::Cell;
use coordinate::Coord;
use futures::{future::join_all, stream::SplitStream, StreamExt};
use packet::Packet;
use player::Player;
use size::Size;
use tokio::sync::Mutex;

use crate::game::{
    config::Config,
    packet::SnakeChange,
    perk::{Generator, Perk},
    player::PlayerId,
    speed::Speed,
    tick::TickManager,
};

mod cell;
pub mod config;
mod coordinate;
mod direction;
mod packet;
mod perk;
mod player;
mod size;
mod speed;
mod tick;

const EXIT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct Game {
    pub name: String,
    pub size: Size,
    pub speed: u8,
    inner: Arc<Mutex<Inner>>,
}

impl Game {
    pub fn new(config: Config) -> Self {
        let mut inner = Inner {
            grid: vec![vec![Cell::Empty; config.size.width as usize]; config.size.height as usize],
            players: HashMap::new(),
            perks: HashMap::new(),
            perk_generator: Generator::new(&config),
            last_leave: Instant::now(),
        };
        for _ in 0..(config.foods as usize) {
            inner.add_perk(config.size, inner.perk_generator.fresh_food());
        }

        Self {
            name: config.name,
            size: config.size,
            speed: config.speed,
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub async fn run(&self) {
        let mut tick_manager = TickManager::new(self.speed);
        let mut allowed_to_walk = Speed::Normal;

        loop {
            let mut inner = self.inner.lock().await;
            if inner.players.is_empty() {
                if inner.last_leave.elapsed() > EXIT_TIMEOUT {
                    break;
                }
                drop(inner);
                tick_manager.wait_for_join().await;
            } else {
                let fastest_snake = inner.walk_snakes(self.size, allowed_to_walk).await;
                drop(inner);
                allowed_to_walk = tick_manager.sleep(fastest_snake).await;
            }
        }
    }

    pub async fn join(&self, socket: WebSocket) {
        let mut inner = self.inner.lock().await;

        let head = inner.safe_place(self.size);
        let (tx, rx) = socket.split();
        let id = rand::random();
        let mut player = Player::new(head, tx);
        let color = player.color;
        player
            .send(Packet::Info(self.size, &self.name, id).message())
            .await;

        // Add player to game.
        inner
            .broadcast_message(Packet::PlayerJoined(id, head, color))
            .await;
        let player = Arc::new(Mutex::new(player));
        inner.players.insert(id, Arc::clone(&player));
        inner.grid[head.y][head.x] = Cell::Occupied(id);

        // Send snakes info.
        let snakes_message = Packet::Snakes(
            join_all(
                inner
                    .players
                    .iter()
                    .map(|(&id, p)| async move { (id, p.lock().await) }),
            )
            .await,
        )
        .message();
        let mut player_lock = player.lock().await;
        player_lock.send(snakes_message).await;

        // Send perks info.
        let perks = inner
            .perks
            .iter()
            .map(|(coord, perk)| (coord.clone(), perk.clone()))
            .collect::<Vec<_>>();
        player_lock.send(Packet::Perks(perks).message()).await;
        drop(player_lock);
        drop(inner);

        // Process events.
        self.player_loop(player, rx).await;
        // Player left from here.

        // Remove and clean player.
        let mut inner = self.inner.lock().await;
        let player = inner.players.remove(&id).unwrap();
        player
            .lock()
            .await
            .body
            .iter()
            .for_each(|&c| inner.grid[c.y][c.x] = Cell::Empty);
        inner.last_leave = Instant::now();
        inner.broadcast_message(Packet::PlayerLeft(id)).await;
    }

    async fn player_loop(&self, player: Arc<Mutex<Player>>, mut rx: SplitStream<WebSocket>) {
        loop {
            let message = match rx.next().await {
                Some(Ok(message)) => message,
                _ => break,
            };

            let Message::Binary(data) = message else {
                break;
            };
            match data[0] {
                0 => player.lock().await.process_event(&data[1..]).await,
                _ => break,
            }
        }
        // Player left the game from here.
    }

    pub async fn player_count(&self) -> usize {
        self.inner.lock().await.players.len()
    }
}

struct Inner {
    grid: Vec<Vec<Cell>>,
    players: HashMap<PlayerId, Arc<Mutex<Player>>>,
    perks: HashMap<Coord, Perk>,
    perk_generator: Generator,
    last_leave: Instant,
}

impl Inner {
    // Order:
    // - free all tails
    // - group next heads by coord
    // - apply heads (queue respawns and perks consuming)
    // - process respawns
    // - consume perks
    async fn walk_snakes(&mut self, size: Size, allowed_to_walk: Speed) -> Speed {
        let walks = join_all(self.players.iter().map(|(&id, p)| async move {
            let mut player = p.lock().await;
            if player.speed() < allowed_to_walk {
                return None;
            }
            player.walk(size).await.map(|cs| (id, Arc::clone(p), cs))
        }))
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let mut need_respawn = Vec::new();
        let mut perk_consumed = Vec::new();
        let mut changes = Vec::with_capacity(walks.len() * 2);
        let mut new_perks = Vec::new();

        // Free all tails.
        for (id, _player, (removed, _new)) in walks.iter() {
            if let Some(removed) = removed {
                self.grid[removed.y][removed.x] = Cell::Empty;
                changes.push(SnakeChange::RemoveTail(*id));
            }
        }

        // Create new heads, handle collisions and apply perks.
        let collisions = walks.iter().fold(
            HashMap::with_capacity(walks.len()),
            |mut acc, (_, _, (_, new))| {
                *acc.entry(new).or_insert(0u8) += 1;
                acc
            },
        );
        for (player_id, player, (_removed, new)) in walks.iter() {
            match &self.grid[new.y][new.x] {
                Cell::Empty => {
                    if *collisions.get(new).unwrap() == 1 {
                        self.grid[new.y][new.x] = Cell::Occupied(*player_id);
                        changes.push(SnakeChange::AddCell(*player_id, *new));
                    } else {
                        need_respawn.push((*player_id, Arc::clone(player)));
                    }
                }
                Cell::Occupied(_) => {
                    need_respawn.push((*player_id, Arc::clone(player)));
                }
                Cell::Perk(perk) => {
                    if *collisions.get(new).unwrap() == 1 {
                        perk_consumed.push((*player_id, Arc::clone(player), perk.clone()));
                        self.grid[new.y][new.x] = Cell::Occupied(*player_id);
                        self.perks.remove(new);
                        changes.push(SnakeChange::AddCell(*player_id, *new));
                    } else {
                        need_respawn.push((*player_id, Arc::clone(player)));
                    }
                }
            }
        }

        // Process respawns and generate perks.
        for (id, player) in need_respawn {
            let mut player = player.lock().await;
            player
                .body
                .iter()
                .skip(1)
                .for_each(|&c| self.grid[c.y][c.x] = Cell::Empty);
            let head = self.safe_place(size);
            player.respawn(head).await;
            self.grid[head.y][head.x] = Cell::Occupied(id);
            changes.push(SnakeChange::Die(id, head));
        }
        for (id, player, perk) in perk_consumed {
            if let Some(additional_change) = perk
                .consume(id, &mut *player.lock().await, &self.perks)
                .await
            {
                if let SnakeChange::AddCell(id, coord) = additional_change {
                    self.grid[coord.y][coord.x] = Cell::Occupied(id);
                    self.perks.remove(&coord);
                }
                changes.push(additional_change);
            }
            if perk.make_spawn_food() {
                for perk in self.perk_generator.next(id) {
                    let coord = self.add_perk(size, perk.clone());
                    new_perks.push((coord, perk));
                }
            }
        }

        if !changes.is_empty() {
            self.broadcast_message(Packet::SnakeChanges(changes)).await;
        }
        if !new_perks.is_empty() {
            self.broadcast_message(Packet::Perks(new_perks)).await;
        }

        join_all(
            self.players
                .values()
                .map(|p| async { p.lock().await.speed() }),
        )
        .await
        .into_iter()
        .max()
        .unwrap_or(Speed::Normal)
    }

    fn safe_place(&self, size: Size) -> Coord {
        iter::repeat_with(|| Coord::random(size))
            .filter(|c| matches!(self.grid[c.y][c.x], Cell::Empty))
            .next()
            .unwrap()
    }

    fn add_perk(&mut self, size: Size, perk: Perk) -> Coord {
        let coord = self.safe_place(size);
        self.grid[coord.y][coord.x] = Cell::Perk(perk.clone());
        self.perks.insert(coord, perk);
        coord
    }

    async fn broadcast_message(&self, packet: Packet<'_>) {
        if self.players.is_empty() {
            return;
        }
        let message = packet.message();
        join_all(self.players.values().map(|p| {
            let message = message.clone();
            async move {
                p.lock().await.send(message).await;
            }
        }))
        .await;
    }
}
