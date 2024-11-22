#[derive(Debug)]
pub enum Compression {
    Lzma(bool),
    BZip2,
    Zlib,
    None,
}
