use derive_more::Display;

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq)]
pub enum Compression {
    Lzma(bool),
    BZip2,
    Zlib,
    None,
}
