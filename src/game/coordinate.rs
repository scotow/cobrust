use std::ops::Add;

use rand::Rng;

use crate::{
    game::{direction::Dir, size::Size},
    misc::PacketSerialize,
};

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
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

impl PacketSerialize for Coord {
    fn push(&self, out: &mut Vec<u8>) {
        (self.x as u16).push(out);
        (self.y as u16).push(out);
    }
}
