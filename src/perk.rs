use crate::player::Player;

pub trait Perk {
    fn consume(&self, player: &mut Player);
    fn as_u8(&self) -> u8;
}

pub struct Food(pub u16);

impl Perk for Food {
    fn consume(&self, player: &mut Player) {
        player.grow(self.0);
    }

    fn as_u8(&self) -> u8 {
        0
    }
}

