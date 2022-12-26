use std::convert::TryFrom;

use crate::game::coordinate::Coord;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn conflict(&self, other: &Self) -> bool {
        use Dir::*;
        if self == other {
            return true;
        }
        match (self, other) {
            (Up, Down) | (Down, Up) => true,
            (Left, Right) | (Right, Left) => true,
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn opposite(&self) -> Self {
        use Dir::*;
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}

impl From<(Coord, Coord)> for Dir {
    // This doesn't work well if the two cells are split by the cyclic world.
    fn from((head, body): (Coord, Coord)) -> Self {
        use Dir::*;
        let x_delta = head.x as isize - body.x as isize;
        let y_delta = head.y as isize - body.y as isize;
        if x_delta.abs() >= y_delta.abs() {
            if x_delta.is_positive() {
                Right
            } else {
                Left
            }
        } else {
            if y_delta.is_positive() {
                Down
            } else {
                Up
            }
        }
    }
}

impl TryFrom<u8> for Dir {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Dir::*;
        Ok(match value {
            0 => Up,
            1 => Down,
            2 => Left,
            3 => Right,
            _ => return Err(()),
        })
    }
}
