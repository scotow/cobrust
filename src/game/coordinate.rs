use std::ops::Add;

use rand::Rng;

use crate::game::{direction::Dir, size::Size};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    pub fn random(size: Size) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(0..size.width as usize),
            y: rng.gen_range(0..size.height as usize),
        }
    }
}

impl Add<(Dir, Size)> for Coord {
    type Output = Self;

    fn add(self, rhs: (Dir, Size)) -> Self::Output {
        match rhs.0 {
            Dir::Up => Coord {
                x: self.x,
                y: (self.y as isize - 1).rem_euclid(rhs.1.height as isize) as usize,
            },
            Dir::Down => Coord {
                x: self.x,
                y: (self.y + 1).rem_euclid(rhs.1.height as usize),
            },
            Dir::Left => Coord {
                x: (self.x as isize - 1).rem_euclid(rhs.1.width as isize) as usize,
                y: self.y,
            },
            Dir::Right => Coord {
                x: (self.x + 1).rem_euclid(rhs.1.width as usize),
                y: self.y,
            },
        }
    }
}
