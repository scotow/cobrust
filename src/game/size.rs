use crate::misc::ToData;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl ToData for Size {
    fn push(&self, out: &mut Vec<u8>) {
        (self.width as u16).push(out);
        (self.height as u16).push(out);
    }
}
