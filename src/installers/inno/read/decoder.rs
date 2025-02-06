use flate2::read::ZlibDecoder;
use liblzma::read::XzDecoder;
use std::io::{Read, Result};

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
