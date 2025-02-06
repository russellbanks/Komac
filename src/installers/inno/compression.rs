use flate2::read::ZlibDecoder;
use liblzma::read::XzDecoder;
use std::io::{Read, Result};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
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

pub enum Decoder<R: Read> {
    Stored(R),
    Zlib(ZlibDecoder<R>),
    LZMA1(XzDecoder<R>),
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::Stored(reader) => reader.read(buf),
            Self::Zlib(reader) => reader.read(buf),
            Self::LZMA1(reader) => reader.read(buf),
        }
    }
}
