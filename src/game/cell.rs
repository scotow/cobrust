use crate::game::{perk::Perk, player::PlayerId};

#[derive(Clone, Debug)]
pub enum Cell {
    Empty,
    Occupied(PlayerId),
    Perk(Perk),
}
