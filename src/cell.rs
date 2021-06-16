use crate::perk::Perk;

pub enum Cell {
    Empty,
    Occupied,
    Perk(Box<dyn Perk + Send + Sync>),
}

impl Cell {
    pub fn as_char(&self) -> char {
        use Cell::*;
        match self {
            Empty => '.',
            Occupied => '#',
            Perk(_) => 'o',
        }
    }

    pub fn as_u8(&self) -> u8 {
        use Cell::*;
        match self {
            Empty => 0,
            Occupied => 1,
            Perk(perk) => perk.as_u8(),
        }
    }
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        Cell::Empty
    }
}