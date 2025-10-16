use std::io::{Read, Result, Seek};

use bzip2::read::BzDecoder;
use flate2::read::ZlibDecoder;
use liblzma::read::XzDecoder;

pub enum Decoder<R: Read + Seek> {
    Lzma(XzDecoder<R>),
    BZip2(BzDecoder<R>),
    Zlib(ZlibDecoder<R>),
    None(R),
}

impl<R: Read + Seek> Decoder<R> {
    pub fn into_inner(self) -> R {
        match self {
            Self::Lzma(reader) => reader.into_inner(),
            Self::BZip2(reader) => reader.into_inner(),
            Self::Zlib(reader) => reader.into_inner(),
            Self::None(reader) => reader,
        }
    }
}

impl<R: Read + Seek> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::Lzma(reader) => reader.read(buf),
            Self::BZip2(reader) => reader.read(buf),
            Self::Zlib(reader) => reader.read(buf),
            Self::None(reader) => reader.read(buf),
        }
    }
}
