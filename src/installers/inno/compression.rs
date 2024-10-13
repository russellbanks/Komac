use std::ops::{Deref, DerefMut};

pub enum Compression {
    Stored(u32),
    Zlib(u32),
    LZMA1(u32),
}

impl Deref for Compression {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Stored(size) | Self::Zlib(size) | Self::LZMA1(size) => size,
        }
    }
}

impl DerefMut for Compression {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Stored(size) | Self::Zlib(size) | Self::LZMA1(size) => size,
        }
    }
}
