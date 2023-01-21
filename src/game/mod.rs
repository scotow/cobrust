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
    player::{BodyId, PlayerId},
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

#[derive(Debug)]
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
            inner.add_perk(config.size, inner.perk_generator.respawnable_food());
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
        let player_id = rand::random();
        let (mut player, body_id) = Player::new(player_id, head, tx);
        let color = player.color;
        player
            .send(Packet::Info(self.size, &self.name, player_id).message())
            .await;

        // Add player to game.
        inner
            .broadcast_message(Packet::PlayerJoined(player_id, body_id, head, color))
            .await;
        let player = Arc::new(Mutex::new(player));
        inner.players.insert(player_id, Arc::clone(&player));
        inner.grid[head.y][head.x] = Cell::Occupied(player_id);

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
        let player = inner.players.remove(&player_id).unwrap();
        for cell in player
            .lock()
            .await
            .bodies_iter()
            .flat_map(|b| b.cells.iter())
        {
            inner.grid[cell.coord.y][cell.coord.x] = Cell::Empty
        }
        inner.last_leave = Instant::now();
        inner.broadcast_message(Packet::PlayerLeft(player_id)).await;
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
            let Some(message_id) = data.get(0) else {
                break;
            };

            match message_id {
                0 => player.lock().await.process_move_event(&data[1..]).await,
                1 => {
                    let (id, new_color) = {
                        let mut player = player.lock().await;
                        (player.id, player.change_color())
                    };
                    self.inner
                        .lock()
                        .await
                        .broadcast_message(Packet::ColorChange(id, new_color))
                        .await;
                }
                _ => break,
            }
        }
        // Player left the game from here.
    }

    pub async fn player_count(&self) -> usize {
        self.inner.lock().await.players.len()
    }
}

#[derive(Debug)]
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
    // - consume perks
    // - process respawns
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
        for (player_id, _player, body_changes) in walks.iter() {
            for (body_id, removed, _new) in body_changes {
                if let Some(removed) = removed {
                    self.grid[removed.coord.y][removed.coord.x] = Cell::Empty;
                    changes.push(SnakeChange::RemoveTail(*player_id, *body_id));
                    if let Some(perk) = &removed.perk {
                        self.grid[removed.coord.y][removed.coord.x] = Cell::Perk(perk.clone());
                        self.perks.insert(removed.coord, perk.clone());
                        new_perks.push((removed.coord, perk.clone()));
                    }
                }
            }
        }

        // Create new heads, handle collisions and queue perks consumption.
        let collisions = walks.iter().flat_map(|w| &w.2).fold(
            HashMap::with_capacity(walks.len()),
            |mut acc, (_, _, new)| {
                *acc.entry(new).or_insert(0u8) += 1;
                acc
            },
        );
        for (player_id, player, body_changes) in walks.iter() {
            for (body_id, _removed, new) in body_changes {
                match &self.grid[new.y][new.x] {
                    Cell::Empty => {
                        if collisions[&new] == 1 {
                            self.grid[new.y][new.x] = Cell::Occupied(*player_id);
                            changes.push(SnakeChange::AddCell(*player_id, *body_id, *new));
                        } else {
                            need_respawn.push((Arc::clone(player), *body_id, false));
                        }
                    }
                    Cell::Occupied(_) => {
                        need_respawn.push((Arc::clone(player), *body_id, false));
                    }
                    Cell::Perk(perk) => {
                        if collisions[&new] == 1 {
                            perk_consumed.push((
                                *player_id,
                                *body_id,
                                Arc::clone(player),
                                perk.clone(),
                            ));
                            self.grid[new.y][new.x] = Cell::Occupied(*player_id);
                            self.perks.remove(new);
                            changes.push(SnakeChange::AddCell(*player_id, *body_id, *new));
                        } else {
                            need_respawn.push((Arc::clone(player), *body_id, false));
                        }
                    }
                }
            }
        }

        // Consume perks and process respawns.
        for (player_id, body_id, player, perk) in perk_consumed {
            let consumption = perk
                .consume(player_id, body_id, &mut *player.lock().await, &self.perks)
                .await;

            if let Some(change) = consumption.snake_change {
                if let SnakeChange::AddCell(player_id, _body_id, coord) = change {
                    self.grid[coord.y][coord.x] = Cell::Occupied(player_id);
                    self.perks.remove(&coord);
                }
                changes.push(change);
            }
            for perk in consumption.additional_perks {
                let coord = self.add_perk(size, perk.clone());
                new_perks.push((coord, perk));
            }
            if let Some(count) = consumption.should_multiply {
                let mut player_lock = player.lock().await;
                for _ in 0..count {
                    let head = self.safe_place(size);
                    let new_body_id = player_lock.add_body(head);
                    self.grid[head.y][head.x] = Cell::Occupied(player_id);
                    changes.push(SnakeChange::AddBody(player_id, new_body_id, head));
                }
            }
            if consumption.should_die {
                need_respawn.push((player, body_id, true));
            }

            if perk.makes_spawn_food() {
                for perk in self.perk_generator.next(player_id) {
                    let coord = self.add_perk(size, perk.clone());
                    new_perks.push((coord, perk));
                }
            }
        }
        for (player, body_id, clear_head) in need_respawn {
            let mut player = player.lock().await;
            self.clear_body(&mut player, body_id, clear_head).await;
            changes.push(SnakeChange::RemoveBody(player.id, body_id));
            if player.bodies_len() == 0 {
                let head = self.safe_place(size);
                let new_body_id = player.add_body(head);
                self.grid[head.y][head.x] = Cell::Occupied(player.id);
                changes.push(SnakeChange::AddBody(player.id, new_body_id, head));
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

    async fn clear_body(&mut self, player: &mut Player, body_id: BodyId, clear_head: bool) {
        let Some(cells) = player.remove_body(body_id).await else {
            return;
        };
        cells
            .into_iter()
            .skip(!clear_head as usize)
            .for_each(|c| self.grid[c.coord.y][c.coord.x] = Cell::Empty);
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
