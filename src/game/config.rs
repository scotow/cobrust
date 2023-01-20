use std::io::{Cursor, Read};

use byteorder::{ReadBytesExt, BE};

use crate::game::size::Size;

pub struct Config {
    pub name: String,
    pub size: Size,
    pub speed: u8,
    pub foods: u16,
    pub food_strength: u16,
    pub reserved_food: bool,
    pub perk_spacing: u16,
    pub reverser: bool,
    pub teleporter: bool,
    pub speed_boost: Option<u16>,
    pub food_frenzy: Option<u8>,
    pub mines_trail: Option<u8>,
    pub multi_snake: bool,
}

impl Config {
    // Read / written by client in the same order of the UI.
    pub fn from_raw(data: &[u8]) -> Option<Self> {
        let mut data = Cursor::new(data);
        let name_size = data.read_u16::<BE>().ok()?;
        let mut name = vec![0; name_size as usize];
        data.read_exact(&mut name).ok()?;
        let name = String::from_utf8(name).ok()?;

        let size = Size {
            width: data.read_u16::<BE>().ok()?,
            height: data.read_u16::<BE>().ok()?,
        };
        let speed = data.read_u8().ok()?;
        let foods = data.read_u16::<BE>().ok()?;
        let food_strength = data.read_u16::<BE>().ok()?;
        let reserved_food = data.read_u8().ok()? > 0;
        let reverser = data.read_u8().ok()? > 0;
        let teleporter = data.read_u8().ok()? > 0;
        let speed_boost = data.read_u16::<BE>().ok()?;
        let food_frenzy = data.read_u8().ok()?;
        let mines_trail = data.read_u8().ok()?;
        let multi_snake = data.read_u8().ok()? > 0;
        let perk_spacing = data.read_u16::<BE>().ok()?;

        Some(Self {
            name,
            size,
            speed,
            foods,
            food_strength,
            reserved_food,
            perk_spacing,
            reverser,
            teleporter,
            speed_boost: (speed_boost > 0).then_some(speed_boost),
            food_frenzy: (food_frenzy > 0).then_some(food_frenzy),
            mines_trail: (mines_trail > 0).then_some(mines_trail),
            multi_snake,
        })
    }

    pub fn is_valid(&self) -> bool {
        (1..=32).contains(&self.name.len())
            && (16..=255).contains(&self.size.width)
            && (16..=255).contains(&self.size.height)
            && (1..=50).contains(&self.speed)
            && (1..=32).contains(&self.foods)
            && (0..=1024).contains(&self.food_strength)
            && (1..=128).contains(&self.perk_spacing)
            && self
                .speed_boost
                .map(|d| (5..=1000).contains(&d))
                .unwrap_or(true)
            && self
                .food_frenzy
                .map(|c| (2..=64).contains(&c))
                .unwrap_or(true)
            && self
                .mines_trail
                .map(|c| (1..=16).contains(&c))
                .unwrap_or(true)
    }
}
