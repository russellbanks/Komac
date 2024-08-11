use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::bail;
use crc32fast::Hasher;
use std::io;
use std::io::Read;

pub struct InnoBlockFilter<R: Read> {
    inner: R,
    buffer: [u8; 4096],
    pos: usize,
    length: usize,
}

impl<R: Read> InnoBlockFilter<R> {
    pub fn new(inner: R) -> Self {
        InnoBlockFilter {
            inner,
            buffer: [0; 4096],
            pos: 0,
            length: 0,
        }
    }

    fn read_chunk(&mut self) -> color_eyre::Result<bool> {
        let Ok(block_crc32) = self.inner.read_u32::<LittleEndian>() else {
            bail!("Unexpected block end")
        };

        self.length = self.inner.read(&mut self.buffer)?;

        if self.length == 0 {
            bail!("Unexpected block end");
        }

        let mut hasher = Hasher::new();
        hasher.update(&self.buffer[..self.length]);
        let actual_crc32 = hasher.finalize();

        if actual_crc32 != block_crc32 {
            bail!("Block CRC32 mismatch");
        }

        self.pos = 0;

        Ok(true)
    }
}

impl<R: Read> Read for InnoBlockFilter<R> {
    fn read(&mut self, dest: &mut [u8]) -> io::Result<usize> {
        let mut nread = 0;
        let mut remaining = dest.len();

        while remaining > 0 {
            if self.pos == self.length {
                match self.read_chunk() {
                    Ok(true) => {}
                    Ok(false) => return Ok(nread),
                    Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
                }
            }

            let to_copy = std::cmp::min(remaining, self.length - self.pos);
            dest[nread..nread + to_copy]
                .copy_from_slice(&self.buffer[self.pos..self.pos + to_copy]);

            self.pos += to_copy;
            nread += to_copy;
            remaining -= to_copy;
        }

        Ok(nread)
    }
}
