use std::mem::size_of;

use async_trait::async_trait;
use byteorder::{BE, WriteBytesExt};

use crate::game::packet::SnakeChange;
use crate::game::player::{Player, PlayerId};
use crate::misc::ToData;

#[async_trait]
pub trait Perk: ToData {
    async fn consume(&self, id: PlayerId, player: &mut Player) -> Option<SnakeChange>;

    fn make_spawn_food(&self) -> bool {
        false
    }
}

pub struct Generator {
    food_strength: u16,
    count: u8,
    previous_consumer: Option<PlayerId>,
    reserved_food: bool,
    reverser: bool,
}

impl Generator {
    pub fn new(food_strength: u16, reserved_food: bool, reverser: bool) -> Self {
        Self {
            food_strength,
            count: 0,
            previous_consumer: None,
            reserved_food,
            reverser,
        }
    }

    pub fn next(&mut self, consumer: PlayerId) -> Vec<Box<dyn Perk + Sync + Send>> {
        self.count = self.count % u8::MAX + 1;
        let mut perks: Vec<Box<dyn Perk + Sync + Send>> = Vec::with_capacity(3);
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
        if self.reverser && self.count % 8 == 0 {
            perks.push(Box::new(Reverser));
        }

        perks
    }

    pub fn fresh_food(&self) -> Food {
        Food(self.food_strength)
    }
}

pub struct Food(pub u16);

#[async_trait]
impl Perk for Food {
    async fn consume(&self, _id: PlayerId, player: &mut Player) -> Option<SnakeChange> {
        player.grow(self.0);
        None
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

#[async_trait]
impl Perk for ReservedFood {
    async fn consume(&self, id: PlayerId, player: &mut Player) -> Option<SnakeChange> {
        if id == self.owner {
            player.grow(self.strength);
        }
        None
    }
}

impl ToData for ReservedFood {
    fn push(&self, out: &mut Vec<u8>) {
        out.reserve(size_of::<u8>() + size_of::<u16>());
        out.push(1);
        out.write_u16::<BE>(self.owner).unwrap();
    }
}

pub struct Reverser;

#[async_trait]
impl Perk for Reverser {
    async fn consume(&self, id: PlayerId, player: &mut Player) -> Option<SnakeChange> {
        player.reverse().await;
        Some(SnakeChange::Reverse(id))
    }
}

impl ToData for Reverser {
    fn push(&self, out: &mut Vec<u8>) {
        out.push(2);
    }
}