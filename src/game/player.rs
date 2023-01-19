use std::{collections::VecDeque, convert::TryFrom};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use rand::{thread_rng, Rng};
use tokio::sync::Mutex;

use crate::game::{coordinate::Coord, direction::Dir, perk::Perk, size::Size, speed::Speed};

const START_SIZE: u16 = 9;
const TRAIL_PERK_SPACING: u16 = 10;

pub(super) type PlayerId = u16;
pub(super) type Color = u16;

#[derive(Debug)]
pub struct Player {
    pub id: PlayerId,
    pub color: Color,
    pub body: VecDeque<BodyCell>,
    direction: Mutex<Direction>,
    growth: u16,
    speed: u16,
    perk_trail: PerkTrail,
    sink: SplitSink<WebSocket, Message>,
}

#[derive(Default, Debug)]
struct Direction {
    current: Option<Dir>,
    queue: VecDeque<Dir>,
}

#[derive(Debug)]
pub struct BodyCell {
    pub coord: Coord,
    pub perk: Option<Perk>,
}

impl BodyCell {
    fn without_perk(coord: Coord) -> Self {
        Self { coord, perk: None }
    }
}

#[derive(Debug)]
struct PerkTrail {
    remaining: u16,
    until_next: u16,
}

impl PerkTrail {
    fn empty() -> Self {
        Self {
            remaining: 0,
            until_next: TRAIL_PERK_SPACING,
        }
    }

    fn add_mines(&mut self, count: u16) {
        self.remaining += count;
    }

    fn next(&mut self, owner: PlayerId) -> Option<Perk> {
        if self.remaining > 0 {
            if self.until_next == 0 {
                self.remaining -= 1;
                self.until_next = TRAIL_PERK_SPACING;
                Some(Perk::new_mine(owner))
            } else {
                self.until_next -= 1;
                None
            }
        } else {
            None
        }
    }
}

impl Player {
    pub fn new(id: PlayerId, head: Coord, tx: SplitSink<WebSocket, Message>) -> Self {
        Self {
            id,
            color: random_color(),
            body: VecDeque::from([BodyCell::without_perk(head)]),
            direction: Mutex::new(Direction::default()),
            growth: START_SIZE,
            speed: 0,
            perk_trail: PerkTrail::empty(),
            sink: tx,
        }
    }

    pub async fn send(&mut self, message: Message) {
        let _ = self.sink.send(message).await;
    }

    pub async fn process_move_event(&self, data: &[u8]) {
        let Some(&id) = data.get(0) else {
            return;
        };
        let Ok(new) = Dir::try_from(id) else {
            return;
        };

        let mut direction = self.direction.lock().await;
        let last = direction.queue.back().copied().or(direction.current);
        if let Some(dir) = last {
            if !dir.conflict(&new) {
                direction.queue.push_back(new);
            }
        } else {
            direction.queue.push_back(new);
        }
    }

    pub async fn walk(&mut self, grid_size: Size) -> Option<(Option<BodyCell>, Coord)> {
        let mut direction = self.direction.lock().await;
        let new_direction = if !direction.queue.is_empty() {
            let dir = direction.queue.pop_front().unwrap();
            direction.current = Some(dir);
            dir
        } else {
            match direction.current {
                Some(dir) => dir,
                None => return None,
            }
        };

        let current_head_coord = self.body.get(0).unwrap().coord;
        let new_head_coord = current_head_coord + (new_direction, grid_size);
        self.body.push_front(BodyCell {
            coord: new_head_coord,
            perk: self.perk_trail.next(self.id),
        });

        let tail = if self.growth >= 1 {
            self.growth -= 1;
            None
        } else {
            Some(self.body.pop_back().unwrap())
        };
        self.speed = self.speed.saturating_sub(1);

        Some((tail, new_head_coord))
    }

    pub fn grow(&mut self, grow: u16) {
        self.growth += grow;
    }

    pub fn speed(&self) -> Speed {
        if self.speed > 0 {
            Speed::Fast
        } else {
            Speed::Normal
        }
    }

    pub fn increase_speed(&mut self, duration: u16) {
        self.speed += duration;
    }

    pub fn increase_mines_count(&mut self, count: u16) {
        self.perk_trail.add_mines(count);
    }

    pub async fn respawn(&mut self, head: Coord) {
        self.body.clear();
        self.body.push_front(BodyCell::without_perk(head));
        let mut direction = self.direction.lock().await;
        direction.current = None;
        direction.queue.clear();
        self.growth = START_SIZE;
        self.speed = 0;
    }

    pub async fn reverse(&mut self) {
        self.body.make_contiguous().reverse();
        let mut direction = self.direction.lock().await;
        if let (Some(head), Some(body)) = (self.body.get(0), self.body.get(1)) {
            direction.current = Some(Dir::from((head.coord, body.coord)));
        } else {
            direction.current = None;
        }
        direction.queue.clear();
    }

    pub async fn teleport(&mut self, coord: Coord) -> bool {
        if self.direction.lock().await.current.is_none() {
            return false;
        }
        self.body.push_front(BodyCell::without_perk(coord));
        true
    }

    pub fn change_color(&mut self) -> Color {
        self.color = random_color();
        self.color
    }
}

fn random_color() -> Color {
    thread_rng().gen_range(0..360)
}
