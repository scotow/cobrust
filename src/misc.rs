#[macro_export]
macro_rules! packet {
    [$($elem:expr),*] => {
        {
            let mut packet = Vec::with_capacity(32);
            $(
                $elem.push(&mut packet);
            )*
            packet
        }
    };
    [$vec:expr; $($elem:expr),*] => {
        {
            $(
                $elem.push(&mut $vec);
            )*
        }
    };
}

pub trait ToData {
    fn push(&self, out: &mut Vec<u8>);
}

impl ToData for u8 {
    fn push(&self, out: &mut Vec<u8>) { out.push(*self) }
}

impl ToData for u16 {
    fn push(&self, out: &mut Vec<u8>) { out.extend_from_slice(&self.to_be_bytes()) }
}

impl ToData for [u8] {
    fn push(&self, out: &mut Vec<u8>) { out.extend_from_slice(self) }
}