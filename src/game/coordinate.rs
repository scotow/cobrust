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
        use Dir::*;
        match rhs.0 {
            Up => Coord {
                x: self.x,
                y: (self.y as isize - 1).rem_euclid(rhs.1.height as isize) as usize,
            },
            Down => Coord {
                x: self.x,
                y: (self.y as isize + 1).rem_euclid(rhs.1.height as isize) as usize,
            },
            Left => Coord {
                x: (self.x as isize - 1).rem_euclid(rhs.1.width as isize) as usize,
                y: self.y,
            },
            Right => Coord {
                x: (self.x as isize + 1).rem_euclid(rhs.1.width as isize) as usize,
                y: self.y,
            },
        }
    }
}
