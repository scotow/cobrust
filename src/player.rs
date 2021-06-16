use crate::coordinate::Coord;
use crate::direction::Dir;
use tokio::sync::Mutex;
use std::collections::VecDeque;
use futures::stream::{SplitSink, SplitStream};
use warp::ws::{Message, WebSocket};
use futures::{StreamExt};
use std::convert::TryFrom;
use crate::size::Size;

#[derive(Debug)]
pub struct Player {
    pub body: VecDeque<Coord>,
    direction: Mutex<Direction>,
    growth: u16,
    pub sink: SplitSink<WebSocket, Message>,
}

#[derive(Debug, Default)]
struct Direction {
    current: Option<Dir>,
    queue: VecDeque<Dir>,
}

impl Player {
    pub fn new(socket: WebSocket, head: Coord) -> (Self, SplitStream<WebSocket>) {
        let (tx, rx) = socket.split();
        (Self {
            body: VecDeque::from(vec![head]),
            direction: Mutex::new(Direction::default()),
            growth: 10,
            sink:tx,
        }, rx)
    }

    pub async fn process(&self, message: Message) {
        let new = match Dir::try_from(message.as_bytes()[1]) {
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
        self.growth = 10;
    }
}