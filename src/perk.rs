use crate::player::Player;

pub trait Perk {
    fn consume(&self, player: &mut Player);
}

pub struct Food;

impl Perk for Food {
    fn consume(&self, player: &mut Player) {
        player.grow(10);
    }
}

