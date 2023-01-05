use std::{collections::HashMap, mem::size_of, sync::Arc};

use async_trait::async_trait;
use byteorder::{WriteBytesExt, BE};
use rand::random;

use crate::{
    game::{
        config::Config,
        coordinate::Coord,
        packet::SnakeChange,
        player::{Player, PlayerId},
    },
    misc::ToData,
};

#[async_trait]
pub trait Perk: ToData {
    async fn consume(
        &self,
        id: PlayerId,
        player: &mut Player,
        perks: &HashMap<Coord, Arc<dyn Perk + Send + Sync>>,
    ) -> Option<SnakeChange>;

    fn make_spawn_food(&self) -> bool {
        false
    }

    fn group_id(&self) -> Option<u16> {
        None
    }
}

pub struct Generator {
    food_consumed: u16,
    food_strength: u16,
    reserved_food: bool,
    previous_consumer: Option<PlayerId>,
    perk_spacing: u16,
    enabled_perks_fn: Vec<fn(&Generator) -> Vec<Arc<dyn Perk + Sync + Send>>>,
}

// Perk ideas:
// - Invisible timer
// - Mines spawn (random, behind tail, or head?), if person take 3 reserved foods in a row
// - Frenzy: spawn N foods or reserved foods at once
// - Multi snakes (like pinball)

impl Generator {
    pub fn new(config: &Config) -> Self {
        Self {
            food_consumed: 0,
            food_strength: config.food_strength,
            reserved_food: config.reserved_food,
            previous_consumer: None,
            perk_spacing: config.perk_spacing,
            enabled_perks_fn: ([
                config.reverser.then_some(Generator::reverser),
                config.teleporter.then_some(Generator::teleporter),
            ]
                as [Option<fn(&Generator) -> Vec<Arc<dyn Perk + Sync + Send>>>; 2])
                .into_iter()
                .flatten()
                .collect(),
        }
    }

    // Spread power spawn evenly (even if some are disabled). Vec<fn>?
    pub fn next(&mut self, consumer: PlayerId) -> Vec<Arc<dyn Perk + Sync + Send>> {
        self.food_consumed = self.food_consumed.wrapping_add(1);
        let mut perks: Vec<Arc<dyn Perk + Sync + Send>> = Vec::with_capacity(3);
        perks.push(Arc::new(Food(self.food_strength)));

        if self.reserved_food {
            if self.previous_consumer.take() == Some(consumer) {
                perks.push(Arc::new(ReservedFood {
                    strength: self.food_strength * 2,
                    owner: consumer,
                }));
            } else {
                self.previous_consumer = Some(consumer);
            }
        }
        if !self.enabled_perks_fn.is_empty() && self.food_consumed % self.perk_spacing == 0 {
            perks.extend(
                self.enabled_perks_fn[(self.food_consumed / self.perk_spacing + 1) as usize
                    % self.enabled_perks_fn.len()](self),
            );
        }

        perks
    }

    pub fn fresh_food(&self) -> Food {
        Food(self.food_strength)
    }

    fn reverser(&self) -> Vec<Arc<dyn Perk + Sync + Send>> {
        vec![Arc::new(Reverser)]
    }

    fn teleporter(&self) -> Vec<Arc<dyn Perk + Sync + Send>> {
        let (id_1, id_2) = (random(), random());
        vec![
            Arc::new(Teleporter {
                self_id: id_1,
                dest_id: id_2,
            }),
            Arc::new(Teleporter {
                self_id: id_2,
                dest_id: id_1,
            }),
        ]
    }
}

pub struct Food(pub u16);

#[async_trait]
impl Perk for Food {
    async fn consume(
        &self,
        _id: PlayerId,
        player: &mut Player,
        _perks: &HashMap<Coord, Arc<dyn Perk + Send + Sync>>,
    ) -> Option<SnakeChange> {
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
    async fn consume(
        &self,
        id: PlayerId,
        player: &mut Player,
        _perks: &HashMap<Coord, Arc<dyn Perk + Send + Sync>>,
    ) -> Option<SnakeChange> {
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
    async fn consume(
        &self,
        id: PlayerId,
        player: &mut Player,
        _perks: &HashMap<Coord, Arc<dyn Perk + Send + Sync>>,
    ) -> Option<SnakeChange> {
        player.reverse().await;
        Some(SnakeChange::Reverse(id))
    }
}

impl ToData for Reverser {
    fn push(&self, out: &mut Vec<u8>) {
        out.push(2);
    }
}

pub struct Teleporter {
    self_id: u16,
    dest_id: u16,
}

#[async_trait]
impl Perk for Teleporter {
    // Handle double consume.
    async fn consume(
        &self,
        id: PlayerId,
        player: &mut Player,
        perks: &HashMap<Coord, Arc<dyn Perk + Send + Sync>>,
    ) -> Option<SnakeChange> {
        // Handle simultaneously consuming.
        let dest_coord = *perks
            .iter()
            .find(|(_, perk)| Some(self.dest_id) == perk.group_id())?
            .0;
        player
            .teleport(dest_coord)
            .await
            .then_some(SnakeChange::AddCell(id, dest_coord))
    }

    fn group_id(&self) -> Option<u16> {
        Some(self.self_id)
    }
}

impl ToData for Teleporter {
    fn push(&self, out: &mut Vec<u8>) {
        out.push(3);
    }
}
