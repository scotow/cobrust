use std::io::{Cursor, Read};

use byteorder::{BE, ReadBytesExt};

use crate::game::size::Size;

pub struct Config {
    pub name: String,
    pub size: Size,
    pub speed: u8,
    pub foods: u16,
    pub food_strength: u16,
    pub reserved_food: bool,
    pub reverser: bool,
}

impl Config {
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

        Some(Self { name, size, speed, foods, food_strength, reserved_food, reverser })
    }

    pub fn is_valid(&self) -> bool {
        (1..=32).contains(&self.name.len()) &&
            (16..=255).contains(&self.size.width) &&
            (16..=255).contains(&self.size.height) &&
            (1..=50).contains(&self.speed) &&
            (1..=32).contains(&self.foods) &&
            (0..=1024).contains(&self.food_strength)
    }
}