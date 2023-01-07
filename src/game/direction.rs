use std::convert::TryFrom;

use crate::game::coordinate::Coord;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn conflict(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }
        match (self, other) {
            (Self::Up, Self::Down) | (Self::Down, Self::Up) => true,
            (Self::Left, Self::Right) | (Self::Right, Self::Left) => true,
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl From<(Coord, Coord)> for Dir {
    fn from((head, body): (Coord, Coord)) -> Self {
        let x_delta = head.x as isize - body.x as isize;
        let y_delta = head.y as isize - body.y as isize;
        match (x_delta, y_delta) {
            (1 | ..=-2, 0) => Self::Right,
            (-1 | 2.., 0) => Self::Left,
            (0, 1 | ..=-2) | (0, 0) => Self::Down,
            (0, -1 | 2..) => Self::Up,
            _ => unreachable!(),
        }
    }
}

impl TryFrom<u8> for Dir {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::Left,
            3 => Self::Right,
            _ => return Err(()),
        })
    }
}
