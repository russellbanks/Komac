use std::{io, io::Read};

use crc32fast::Hasher;

pub struct Crc32Reader<R: Read> {
    inner: R,
    hasher: Hasher,
}

impl<R: Read> Crc32Reader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: Hasher::new(),
        }
    }

    /// Provides mutable access to the inner reader without affecting the hasher
    pub const fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Finalize the hash state and return the computed CRC32 value
    pub fn finalize(self) -> u32 {
        self.hasher.finalize()
    }
}

impl<R: Read> Read for Crc32Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;
        self.hasher.update(&buf[..bytes_read]);
        Ok(bytes_read)
    }
}
