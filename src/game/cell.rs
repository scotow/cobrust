use std::sync::Arc;

use crate::game::{perk::Perk, player::PlayerId};

#[derive(Clone)]
pub enum Cell {
    Empty,
    Occupied(PlayerId),
    Perk(Arc<Box<dyn Perk + Send + Sync>>),
}
