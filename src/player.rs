use crate::coord::{Coord, Dir};
use tokio::sync::Mutex;
use std::collections::VecDeque;
use futures::stream::{SplitSink, SplitStream};
use warp::ws::{Message, WebSocket};
use futures::{StreamExt};
use rand::Rng;

#[derive(Debug)]
pub struct Player {
    pub id: usize,
    pub body: Mutex<VecDeque<Coord>>,
    direction: Mutex<Option<Dir>>,
    directions_queue: Mutex<VecDeque<Dir>>,
    growth: Mutex<u16>,
    pub sink: Mutex<SplitSink<WebSocket, Message>>,
}

impl Player {
    pub fn new(socket: WebSocket) -> (Self, SplitStream<WebSocket>, Coord) {
        let (tx, rx) = socket.split();
        let mut rng = rand::thread_rng();
        let head = Coord { x: rng.gen_range(0..16), y: rng.gen_range(0..16) };
        (Self {
            id: rng.gen(),
            body: Mutex::new(VecDeque::from(vec![head])),
            direction: Mutex::new(None),
            directions_queue: Mutex::new(VecDeque::with_capacity(8)),
            growth: Mutex::new(5),
            sink: Mutex::new(tx),
        }, rx, head)
    }

    pub async fn listen(&self, mut rx: SplitStream<WebSocket>) {
        while let Some(Ok(message)) = rx.next().await {
            if message.is_close() {
                break
            }

            let new = match message.as_bytes()[1] {
                0 => Dir { x: 0, y: -1 },
                1 => Dir { x: 0, y: 1 },
                2 => Dir { x: -1, y: 0 },
                3 => Dir { x: 1, y: 0 },
                _ => return
            };

            let mut queue = self.directions_queue.lock().await;
            let last = if let Some(dir) = queue.back().copied() {
                Some(dir)
            } else {
                *self.direction.lock().await
            };
            if let Some(dir) = last {
                if !dir.conflict(&new) {
                    queue.push_back(new);
                }
            } else {
                queue.push_back(new);
            }
        }
    }

    pub async fn walk(&self) -> Option<(Option<Coord>, Coord)> {
        let mut current_direction = self.direction.lock().await;
        let mut queue = self.directions_queue.lock().await;
        let new_direction = if !queue.is_empty() {
            let dir = queue.pop_front().unwrap();
            *current_direction = Some(dir);
            dir
        } else {
            match *current_direction {
                Some(dir) => dir,
                None => return None,
            }
        };

        let mut body = self.body.lock().await;
        let current_head = body.get(0).unwrap();
        let new_head = Coord {
            x: (current_head.x as isize + new_direction.x as isize).rem_euclid(16) as usize,
            y: (current_head.y as isize + new_direction.y as isize).rem_euclid(16) as usize,
        };
        body.push_front(new_head);

        let mut growth = self.growth.lock().await;
        let tail = if *growth >= 1 {
            *growth = *growth - 1;
            None
        } else {
            Some(body.pop_back().unwrap())
        };

        Some((tail, new_head))
    }
}