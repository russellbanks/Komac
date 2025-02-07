use bzip2::read::BzDecoder;
use flate2::read::DeflateDecoder;
use liblzma::read::XzDecoder;
use std::io::Read;

pub enum Decoder<R: Read> {
    Lzma(XzDecoder<R>),
    BZip2(BzDecoder<R>),
    Zlib(DeflateDecoder<R>),
    None(R),
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Decoder::Lzma(reader) => reader.read(buf),
            Decoder::BZip2(reader) => reader.read(buf),
            Decoder::Zlib(reader) => reader.read(buf),
            Decoder::None(reader) => reader.read(buf),
        }
    }
}
