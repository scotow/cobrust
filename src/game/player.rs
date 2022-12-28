use std::{collections::VecDeque, convert::TryFrom};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use rand::{thread_rng, Rng};
use tokio::sync::Mutex;

use crate::game::{coordinate::Coord, direction::Dir, size::Size};

const COLOR_GAP: u16 = 60;
const START_SIZE: u16 = 9;

pub(super) type PlayerId = u16;

#[derive(Debug)]
pub struct Player {
    pub body: VecDeque<Coord>,
    direction: Mutex<Direction>,
    growth: u16,
    pub color: (u16, u16),
    sink: SplitSink<WebSocket, Message>,
}

#[derive(Debug, Default)]
struct Direction {
    current: Option<Dir>,
    queue: VecDeque<Dir>,
}

impl Player {
    pub fn new(head: Coord, tx: SplitSink<WebSocket, Message>) -> Self {
        let head_color = thread_rng().gen_range(0..360);
        Self {
            body: VecDeque::from(vec![head]),
            direction: Mutex::new(Direction::default()),
            growth: START_SIZE,
            color: (
                head_color,
                head_color + COLOR_GAP + thread_rng().gen_range(0..360 - COLOR_GAP * 2),
            ),
            sink: tx,
        }
    }

    pub async fn send(&mut self, message: Message) {
        let _ = self.sink.send(message).await;
    }

    pub async fn process(&self, data: &[u8]) {
        let new = match Dir::try_from(data[0]) {
            Ok(dir) => dir,
            Err(_) => return,
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

    pub async fn walk(&mut self, grid_size: Size) -> Option<(Option<Coord>, Coord)> {
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

        let current_head = *self.body.get(0).unwrap();
        let new_head = current_head + (new_direction, grid_size);
        self.body.push_front(new_head);

        let tail = if self.growth >= 1 {
            self.growth -= 1;
            None
        } else {
            Some(self.body.pop_back().unwrap())
        };

        Some((tail, new_head))
    }

    pub fn grow(&mut self, grow: u16) {
        self.growth += grow;
    }

    pub async fn respawn(&mut self, head: Coord) {
        self.body.clear();
        self.body.push_front(head);
        let mut direction = self.direction.lock().await;
        direction.current = None;
        direction.queue.clear();
        self.growth = START_SIZE;
    }

    pub async fn reverse(&mut self) {
        self.body.make_contiguous().reverse();
        let mut direction = self.direction.lock().await;
        if let (Some(&head), Some(&body)) = (self.body.get(0), self.body.get(1)) {
            direction.current = Some(Dir::from((head, body)));
        } else {
            direction.current = None;
        }
        direction.queue.clear();
    }

    pub async fn teleport(&mut self, coord: Coord) -> bool {
        if self.direction.lock().await.current.is_none() {
            return false;
        }
        self.body.push_front(coord);
        true
    }
}
