use std::ops::{Add, Rem};
use crate::direction::Dir;
use crate::size::Size;

#[derive(Copy, Clone, Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
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
                y: self.y
            },
        }
    }
}

impl Rem<Size> for Coord {
    type Output = Self;

    fn rem(self, rhs: Size) -> Self::Output {
        Coord {
            x: self.x.rem_euclid(rhs.width),
            y: self.y.rem_euclid(rhs.height),
        }
    }
}