use crate::game::perk::Perk;
use std::sync::Arc;
use crate::game::player::PlayerId;

pub enum Cell {
    Empty,
    Occupied(PlayerId),
    Perk(Arc<Box<dyn Perk + Send + Sync>>),
}