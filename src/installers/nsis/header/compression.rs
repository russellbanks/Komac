use derive_more::Display;

#[derive(Copy, Clone, Debug, Display)]
pub enum Compression {
    Lzma(bool),
    BZip2,
    Zlib,
    None,
}
