use derive_more::Display;

#[derive(Debug, Display)]
pub enum Compression {
    Lzma(bool),
    BZip2,
    Zlib,
    None,
}
