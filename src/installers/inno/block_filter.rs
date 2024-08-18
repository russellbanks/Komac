use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::bail;
use color_eyre::Result;
use crc32fast::Hasher;
use std::cmp::min;
use std::io;
use std::io::Read;

pub const INNO_BLOCK_SIZE: u16 = 1 << 12;

pub struct InnoBlockFilter<R: Read> {
    inner: R,
    buffer: [u8; INNO_BLOCK_SIZE as usize],
    pos: usize,
    length: usize,
}

impl<R: Read> InnoBlockFilter<R> {
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: [0; INNO_BLOCK_SIZE as usize],
            pos: 0,
            length: 0,
        }
    }

    fn read_chunk(&mut self) -> Result<bool> {
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
        let mut read_count = 0;
        let mut remaining = dest.len();

        while remaining > 0 {
            if self.pos == self.length {
                match self.read_chunk() {
                    Ok(true) => {}
                    Ok(false) => return Ok(read_count),
                    Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
                }
            }

            let to_copy = min(remaining, self.length - self.pos);
            dest[read_count..read_count + to_copy]
                .copy_from_slice(&self.buffer[self.pos..self.pos + to_copy]);

            self.pos += to_copy;
            read_count += to_copy;
            remaining -= to_copy;
        }

        Ok(read_count)
    }
}
