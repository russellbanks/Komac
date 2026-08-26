use std::io::{Read, Result, Seek};

use bzip2::read::BzDecoder;
use flate2::read::ZlibDecoder;
use lzma_rust2::LzmaReader;

use super::LzmaStreamHeader;

pub enum Decoder<R: Read + Seek> {
    Lzma(LzmaReader<R>),
    BZip2(BzDecoder<R>),
    Zlib(ZlibDecoder<R>),
    None(R),
}

impl<R: Read + Seek> Decoder<R> {
    /// Creates a new LZMA1 decoder from a reader and an [`LzmaStreamHeader`].
    pub fn new_lzma1(reader: R, header: LzmaStreamHeader) -> lzma_rust2::Result<Self> {
        LzmaReader::new_with_props(
            reader,
            u64::MAX,
            header.props(),
            header.dictionary_size(),
            None,
        )
        .map(|reader| Self::Lzma(reader))
    }
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
