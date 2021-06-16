use crate::player::Player;

pub trait Perk {
    fn consume(&self, player: &mut Player);
    fn as_u8(&self) -> u8;
}

pub struct Food;

impl Perk for Food {
    fn consume(&self, player: &mut Player) {
        player.grow(10);
    }

    fn as_u8(&self) -> u8 {
        2
    }
}

