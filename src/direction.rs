use std::convert::TryFrom;
use std::ops::Add;
use crate::coordinate::Coord;

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
            return true
        }
        match (self, other) {
            (Up, Down) | (Down, Up) => true,
            (Left, Right) | (Right, Left) => true,
            _ => false
        }
    }
}

impl TryFrom<u8> for Dir {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Dir::*;
        Ok(
            match value {
                0 => Up,
                1 => Down,
                2 => Left,
                3 => Right,
                _ => return Err(())
            }
        )
    }
}