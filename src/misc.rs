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
    [cap $size:expr; $($elem:expr),*] => {
        {
            let mut packet = Vec::with_capacity($size);
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

pub trait PacketSerialize {
    fn push(&self, out: &mut Vec<u8>);
}

impl PacketSerialize for u8 {
    fn push(&self, out: &mut Vec<u8>) {
        out.push(*self)
    }
}

impl PacketSerialize for u16 {
    fn push(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_be_bytes())
    }
}

impl PacketSerialize for [u8] {
    fn push(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self)
    }
}

impl<T: PacketSerialize> PacketSerialize for Option<T> {
    fn push(&self, out: &mut Vec<u8>) {
        out.push(self.is_some() as u8);
        if let Some(inner) = self {
            inner.push(out);
        }
    }
}
