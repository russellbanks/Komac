#[derive(Debug)]
pub enum Compression {
    Lzma,
    BZip2,
    Zlib,
    None,
}
