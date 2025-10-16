#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Compression {
    Lzma(bool),
    BZip2,
    Zlib,
    None,
}
