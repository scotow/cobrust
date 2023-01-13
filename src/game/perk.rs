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

#[derive(Clone, Debug)]
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

    pub fn new_mine(owner: PlayerId) -> Self {
        Self::new(PerkKind::Mine(owner))
    }

    pub async fn consume(
        &self,
        player_id: PlayerId,
        player: &mut Player,
        perks: &HashMap<Coord, Perk>,
    ) -> PerkConsumption {
        let mut consumption = PerkConsumption::default();
        match self.kind {
            PerkKind::Food(strength, _) => player.grow(strength),
            PerkKind::ReservedFood { strength, owner } => {
                if owner == player_id {
                    player.grow(strength);
                }
            }
            PerkKind::Reverser => {
                player.reverse().await;
                consumption.snake_change = Some(SnakeChange::Reverse(player_id));
            }
            PerkKind::Teleporter => {
                consumption.snake_change = async {
                    let departure = player.body.front()?.coord;
                    let arrival = *perks
                        .iter()
                        .find(|(&coord, perk)| {
                            perk.group_id == self.group_id && coord != departure
                        })?
                        .0;
                    player
                        .teleport(arrival)
                        .await
                        .then_some(SnakeChange::AddCell(player_id, arrival))
                }
                .await;
            }
            PerkKind::SpeedBoost(duration) => {
                player.increase_speed(duration);
            }
            PerkKind::FoodFrenzy { count, strength } => {
                consumption.additional_perks.extend(vec![
                    Perk::new(PerkKind::Food(strength, false));
                    count as usize
                ]);
            }
            PerkKind::MinesTrail(count) => {
                player.increase_mines_count(count as u16);
            }
            PerkKind::Mine(owner) => {
                if owner != player_id {
                    consumption.should_die = true;
                }
            }
        }
        consumption
    }

    pub fn makes_spawn_food(&self) -> bool {
        matches!(self.kind, PerkKind::Food(_, true))
    }
}

impl PacketSerialize for Perk {
    fn push(&self, out: &mut Vec<u8>) {
        out.push(self.kind.enum_index() as u8);
        match self.kind {
            PerkKind::ReservedFood { owner, .. } => out.write_u16::<BE>(owner).unwrap(),
            PerkKind::Mine(owner) => out.write_u16::<BE>(owner).unwrap(),
            _ => (),
        }
    }
}

#[derive(EnumIndex, Clone, Debug)]
enum PerkKind {
    Food(u16, bool),
    ReservedFood { strength: u16, owner: PlayerId },
    Reverser,
    Teleporter,
    SpeedBoost(u16),
    FoodFrenzy { count: u8, strength: u16 },
    MinesTrail(u8),
    Mine(PlayerId),
}

#[derive(Default, Debug)]
pub struct PerkConsumption {
    pub snake_change: Option<SnakeChange>,
    pub additional_perks: Vec<Perk>,
    pub should_die: bool,
}

pub struct Generator {
    food_consumed: u16,
    food_strength: u16,
    reserved_food: bool,
    previous_consumer: Option<PlayerId>,
    perk_spacing: u16,
    speed_boost: Option<u16>,
    food_frenzy: Option<u8>,
    mines_trail: Option<u8>,
    enabled_perks_fn: Vec<fn(&Generator) -> Vec<Perk>>,
}

// Perk ideas:
// - Invisible timer
// - Multi snakes (like pinball)

type PerkGeneratorFn = fn(&Generator) -> Vec<Perk>;

impl Generator {
    pub fn new(config: &Config) -> Self {
        let mut enabled_perks_fn = [
            config
                .reverser
                .then_some(Generator::reverser as PerkGeneratorFn),
            config
                .teleporter
                .then_some(Generator::teleporter as PerkGeneratorFn),
            config
                .speed_boost
                .map(|_| Generator::speed_boost as PerkGeneratorFn),
            config
                .food_frenzy
                .map(|_| Generator::food_frenzy as PerkGeneratorFn),
            config
                .mines_trail
                .map(|_| Generator::mines_trail as PerkGeneratorFn),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        if !enabled_perks_fn.is_empty() {
            enabled_perks_fn.rotate_right(1);
        }

        Self {
            food_consumed: 0,
            food_strength: config.food_strength,
            reserved_food: config.reserved_food,
            previous_consumer: None,
            perk_spacing: config.perk_spacing,
            speed_boost: config.speed_boost,
            food_frenzy: config.food_frenzy,
            mines_trail: config.mines_trail,
            enabled_perks_fn,
        }
    }

    pub fn next(&mut self, consumer: PlayerId) -> Vec<Perk> {
        self.food_consumed = self.food_consumed.wrapping_add(1);
        let mut perks = Vec::with_capacity(3);
        perks.push(self.respawnable_food());

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
            let next_perk_fn_idx =
                (self.food_consumed / self.perk_spacing) as usize % self.enabled_perks_fn.len();
            perks.extend(self.enabled_perks_fn[next_perk_fn_idx](self));
        }

        perks
    }

    pub fn respawnable_food(&self) -> Perk {
        Perk::new(PerkKind::Food(self.food_strength, true))
    }

    fn reverser(&self) -> Vec<Perk> {
        vec![Perk::new(PerkKind::Reverser)]
    }

    fn teleporter(&self) -> Vec<Perk> {
        vec![Perk::new(PerkKind::Teleporter); 2]
    }

    fn speed_boost(&self) -> Vec<Perk> {
        vec![Perk::new(PerkKind::SpeedBoost(self.speed_boost.unwrap()))]
    }

    fn food_frenzy(&self) -> Vec<Perk> {
        vec![Perk::new(PerkKind::FoodFrenzy {
            count: self.food_frenzy.unwrap(),
            strength: self.food_strength,
        })]
    }

    fn mines_trail(&self) -> Vec<Perk> {
        vec![Perk::new(PerkKind::MinesTrail(self.mines_trail.unwrap()))]
    }
}
