use crate::perk::Perk;
use std::sync::Arc;

pub enum Cell {
    Empty,
    Occupied,
    Perk(Arc<Box<dyn Perk + Send + Sync>>),
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        Cell::Empty
    }
}