use std::mem::size_of;

use byteorder::{BE, WriteBytesExt};

use crate::game::player::{Player, PlayerId};
use crate::misc::ToData;

pub trait Perk: ToData {
    fn consume(&self, id: PlayerId, player: &mut Player);

    fn make_spawn_food(&self) -> bool {
        false
    }
}

pub struct Generator {
    food_strength: u16,
    count: u8,
    previous_consumer: Option<PlayerId>,
    reserved_food: bool,
}

impl Generator {
    pub fn new(food_strength: u16, reserved_food: bool) -> Self {
        Self {
            food_strength,
            count: 0,
            previous_consumer: None,
            reserved_food,
        }
    }

    pub fn next(&mut self, consumer: PlayerId) -> Vec<Box<dyn Perk + Sync + Send>> {
        self.count += 1;
        let mut perks: Vec<Box<dyn Perk + Sync + Send>> = Vec::with_capacity(2);
        perks.push(Box::new(Food(self.food_strength)));

        if self.reserved_food {
            if self.previous_consumer == Some(consumer) {
                perks.push(Box::new(ReservedFood {
                    strength: self.food_strength * 2,
                    owner: consumer,
                }));
                self.previous_consumer = None;
            } else {
                self.previous_consumer = Some(consumer);
            }
        }

        perks
    }

    pub fn fresh_food(&self) -> Food {
        Food(self.food_strength)
    }
}

pub struct Food(pub u16);

impl Perk for Food {
    fn consume(&self, _id: PlayerId, player: &mut Player) {
        player.grow(self.0);
    }

    fn make_spawn_food(&self) -> bool {
        true
    }
}

impl ToData for Food {
    fn push(&self, out: &mut Vec<u8>) {
        out.push(0);
    }
}

pub struct ReservedFood {
    pub strength: u16,
    pub owner: PlayerId,
}

impl Perk for ReservedFood {
    fn consume(&self, id: PlayerId, player: &mut Player) {
        if id == self.owner {
            player.grow(self.strength);
        }
    }
}

impl ToData for ReservedFood {
    fn push(&self, out: &mut Vec<u8>) {
        out.reserve(size_of::<u8>() + size_of::<u16>());
        out.push(1);
        out.write_u16::<BE>(self.owner).unwrap();
    }
}

