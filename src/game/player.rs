use std::{collections::VecDeque, convert::TryFrom};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use rand::{random, thread_rng, Rng};
use tokio::sync::Mutex;

use crate::game::{coordinate::Coord, direction::Dir, perk::Perk, size::Size, speed::Speed};

const START_SIZE: u16 = 9;
const TRAIL_PERK_SPACING: u16 = 10;
const COLOR_GAP: u16 = 60;

pub(super) type PlayerId = u16;
pub(super) type BodyId = u16;
pub(super) type Color = u16;

#[derive(Debug)]
pub struct Player {
    pub id: PlayerId,
    pub color: Color,
    bodies: Vec<Body>,
    direction: Mutex<Direction>,
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
pub struct Body {
    pub id: BodyId,
    pub cells: VecDeque<BodyCell>,
    growth: u16,
}

impl Body {
    fn new(head: Coord) -> Self {
        Self {
            id: random(),
            cells: VecDeque::from([BodyCell::without_perk(head)]),
            growth: START_SIZE,
        }
    }
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
    pub fn new(id: PlayerId, head: Coord, tx: SplitSink<WebSocket, Message>) -> (Self, BodyId) {
        let body = Body::new(head);
        let body_id = body.id;
        (
            Self {
                id,
                color: thread_rng().gen_range(0..360),
                bodies: vec![body],
                direction: Mutex::new(Direction::default()),
                speed: 0,
                perk_trail: PerkTrail::empty(),
                sink: tx,
            },
            body_id,
        )
    }

    pub async fn send(&mut self, message: Message) {
        let _ = self.sink.send(message).await;
    }

    pub fn add_body(&mut self, head: Coord) -> BodyId {
        let body = Body::new(head);
        let id = body.id;
        self.bodies.push(body);
        id
    }

    pub async fn remove_body(&mut self, id: BodyId) -> Option<VecDeque<BodyCell>> {
        let removed = self
            .bodies
            .remove(self.bodies.iter().position(|body| body.id == id)?);
        if self.bodies.is_empty() {
            let mut direction = self.direction.lock().await;
            direction.current = None;
            direction.queue.clear();
            self.speed = 0;
            self.perk_trail = PerkTrail::empty();
        }
        Some(removed.cells)
    }

    pub fn get_body(&self, id: BodyId) -> Option<&Body> {
        self.bodies.iter().find(|b| b.id == id)
    }

    pub fn bodies_len(&self) -> usize {
        self.bodies.len()
    }

    pub fn bodies_iter(&self) -> impl Iterator<Item = &Body> {
        self.bodies.iter()
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

    pub async fn walk(
        &mut self,
        grid_size: Size,
    ) -> Option<Vec<(BodyId, Option<BodyCell>, Coord)>> {
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

        let mut changes = Vec::with_capacity(self.bodies.len());
        let mine = self.perk_trail.next(self.id);
        for body in &mut self.bodies {
            let current_head_coord = body.cells.get(0).unwrap().coord;
            let new_head_coord = current_head_coord + (new_direction, grid_size);

            body.cells.push_front(BodyCell {
                coord: new_head_coord,
                perk: mine.clone(),
            });
            let tail = if body.growth >= 1 {
                body.growth -= 1;
                None
            } else {
                Some(body.cells.pop_back().unwrap())
            };
            changes.push((body.id, tail, new_head_coord));
        }
        self.speed = self.speed.saturating_sub(1);

        Some(changes)
    }

    pub fn grow(&mut self, grow: u16) {
        for body in &mut self.bodies {
            body.growth += grow;
        }
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

    pub async fn reverse(&mut self) {
        if self.bodies.is_empty() {
            // Should never happen.
            return;
        }

        for body in &mut self.bodies {
            body.cells.make_contiguous().reverse();
        }
        let mut direction = self.direction.lock().await;
        if let (Some(head), Some(body)) = (self.bodies[0].cells.get(0), self.bodies[0].cells.get(1))
        {
            direction.current = Some(Dir::from((head.coord, body.coord)));
        } else {
            direction.current = None;
        }
        direction.queue.clear();
    }

    pub async fn teleport(&mut self, body_id: BodyId, coord: Coord) -> bool {
        if self.direction.lock().await.current.is_none() {
            return false;
        }
        let Some(body) = self.bodies.iter_mut().find(|b| b.id == body_id) else {
            return false;
        };
        body.cells.push_front(BodyCell::without_perk(coord));
        true
    }

    pub fn change_color(&mut self) -> Color {
        self.color =
            (self.color + COLOR_GAP + thread_rng().gen_range(0..360 - COLOR_GAP * 2)) % 360;
        self.color
    }
}
