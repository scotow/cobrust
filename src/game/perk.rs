use std::{collections::HashMap, vec};

use byteorder::{WriteBytesExt, BE};
use enum_index::EnumIndex;
use enum_index_derive::EnumIndex;
use rand::random;

use crate::{
    game::{
        config::Config,
        coordinate::Coord,
        packet::SnakeChange,
        player::{Player, PlayerId},
    },
    misc::PacketSerialize,
};

const SPEED_BOOST_DURATION: u16 = 100;

#[derive(Clone)]
pub struct Perk {
    group_id: u16,
    kind: PerkKind,
}

impl Perk {
    fn new(kind: PerkKind) -> Self {
        Self {
            group_id: random(),
            kind,
        }
    }

    pub async fn consume(
        &self,
        player_id: PlayerId,
        player: &mut Player,
        perks: &HashMap<Coord, Perk>,
    ) -> Option<SnakeChange> {
        let mut change = None;
        match self.kind {
            PerkKind::Food(strength) => player.grow(strength),
            PerkKind::ReservedFood { strength, owner } => {
                if owner == player_id {
                    player.grow(strength);
                }
            }
            PerkKind::Reverser => {
                player.reverse().await;
                change = Some(SnakeChange::Reverse(player_id));
            }
            PerkKind::Teleporter => {
                let departure = *player.body.front()?;
                let arrival = *perks
                    .iter()
                    .find(|(&coord, perk)| perk.group_id == self.group_id && coord != departure)?
                    .0;
                change = player
                    .teleport(arrival)
                    .await
                    .then_some(SnakeChange::AddCell(player_id, arrival))
            }
            PerkKind::SpeedBoost => {
                player.increase_speed(SPEED_BOOST_DURATION);
            }
        }
        change
    }

    pub fn make_spawn_food(&self) -> bool {
        matches!(self.kind, PerkKind::Food(_))
    }
}

impl PacketSerialize for Perk {
    fn push(&self, out: &mut Vec<u8>) {
        out.push(self.kind.enum_index() as u8);
        match self.kind {
            PerkKind::ReservedFood { owner, .. } => out.write_u16::<BE>(owner).unwrap(),
            _ => (),
        }
    }
}

#[derive(EnumIndex, Clone)]
enum PerkKind {
    Food(u16),
    ReservedFood { strength: u16, owner: PlayerId },
    Reverser,
    Teleporter,
    SpeedBoost,
}

pub struct Generator {
    food_consumed: u16,
    food_strength: u16,
    reserved_food: bool,
    previous_consumer: Option<PlayerId>,
    perk_spacing: u16,
    enabled_perks_fn: Vec<fn(&Generator) -> Vec<Perk>>,
}

// // Perk ideas:
// // - Invisible timer
// // - Mines spawn (random, behind tail, or head?), if person take 3 reserved foods in a row
// // - Frenzy: spawn N foods or reserved foods at once
// // - Multi snakes (like pinball)

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
                Some(Generator::speed_boost),
            ] as [Option<fn(&Generator) -> Vec<Perk>>; 3])
                .into_iter()
                .flatten()
                .collect(),
        }
    }

    pub fn next(&mut self, consumer: PlayerId) -> Vec<Perk> {
        self.food_consumed = self.food_consumed.wrapping_add(1);
        let mut perks = Vec::with_capacity(3);
        perks.push(Perk::new(PerkKind::Food(self.food_strength)));

        if self.reserved_food {
            if self.previous_consumer.take() == Some(consumer) {
                perks.push(Perk::new(PerkKind::ReservedFood {
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

    pub fn fresh_food(&self) -> Perk {
        Perk::new(PerkKind::Food(self.food_strength))
    }

    fn reverser(&self) -> Vec<Perk> {
        vec![Perk::new(PerkKind::Reverser)]
    }

    fn teleporter(&self) -> Vec<Perk> {
        vec![Perk::new(PerkKind::Teleporter); 2]
    }

    fn speed_boost(&self) -> Vec<Perk> {
        vec![Perk::new(PerkKind::SpeedBoost)]
    }
}
