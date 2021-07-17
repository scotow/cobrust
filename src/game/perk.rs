use crate::game::player::Player;
use crate::misc::ToData;
use byteorder::{WriteBytesExt, BE};

pub trait Perk: ToData {
    fn consume(&self, player: (u16, &mut Player));

    fn make_spawn_food(&self) -> bool {
        false
    }
}

pub struct Generator {
    food_strength: u16,
    count: u8,
    previous_consumer: Option<u16>,
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

    pub fn next(&mut self, consumer: u16) -> Vec<Box<dyn Perk + Sync + Send>> {
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
    fn consume(&self,  player: (u16, &mut Player)) {
        player.1.grow(self.0);
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
    pub owner: u16,
}

impl Perk for ReservedFood {
    fn consume(&self,  player: (u16, &mut Player)) {
        if player.0 == self.owner {
            player.1.grow(self.strength);
        }
    }
}

impl ToData for ReservedFood {
    fn push(&self, out: &mut Vec<u8>) {
        out.reserve(3);
        out.push(1);
        out.write_u16::<BE>(self.owner).unwrap();
    }
}

