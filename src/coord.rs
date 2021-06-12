#[derive(Copy, Clone, Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Dir {
    pub x: i8,
    pub y: i8,
}

impl Dir {
    pub fn conflict(&self, other: &Self) -> bool {
        if self.x != 0 && other.x != 0 { return true }
        if self.y != 0 && other.y != 0 { return true }
        false
    }
}