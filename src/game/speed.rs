use std::ops::{AddAssign, BitOr};

#[derive(Copy, Clone, Debug)]
pub enum GameTick {
    Normal,
    SpedUp(bool),
}

impl AddAssign<Speed> for GameTick {
    fn add_assign(&mut self, rhs: Speed) {
        *self = match (*self, rhs) {
            (_, Speed::Normal) => Self::Normal,
            (Self::Normal, Speed::Fast) => Self::SpedUp(true),
            (Self::SpedUp(t), Speed::Fast) => Self::SpedUp(!t),
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Speed {
    Normal,
    Fast,
}

impl BitOr for Speed {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        if self == Speed::Fast || rhs == Speed::Fast {
            Self::Fast
        } else {
            Self::Normal
        }
    }
}

impl From<GameTick> for Speed {
    fn from(value: GameTick) -> Self {
        match value {
            GameTick::Normal | GameTick::SpedUp(false) => Self::Normal,
            GameTick::SpedUp(true) => Self::Fast,
        }
    }
}
