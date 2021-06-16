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
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        Cell::Empty
    }
}