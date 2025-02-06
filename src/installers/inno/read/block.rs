use crate::installers::inno::compression::Compression;
use crate::installers::inno::read::crc32::Crc32Reader;
use crate::installers::inno::version::KnownVersion;
use crate::installers::inno::InnoError;
use byteorder::{ReadBytesExt, LE};
use std::cmp::min;
use std::io::{Error, ErrorKind, Read, Result};

pub const INNO_BLOCK_SIZE: u16 = 1 << 12;

pub struct InnoBlockReader<R: Read> {
    inner: R,
    buffer: [u8; INNO_BLOCK_SIZE as usize],
    pos: usize,
    length: usize,
}

impl<R: Read> InnoBlockReader<R> {
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: [0; INNO_BLOCK_SIZE as usize],
            pos: 0,
            length: 0,
        }
    }

    pub fn read_header(reader: &mut R, version: &KnownVersion) -> Result<Compression> {
        let expected_crc32 = reader.read_u32::<LE>()?;

        let mut actual_crc32 = Crc32Reader::new(reader);

        let compression = if *version >= (4, 0, 9) {
            let size = actual_crc32.read_u32::<LE>()?;
            let compressed = actual_crc32.read_u8()? != 0;

            if compressed {
                if *version >= (4, 1, 6) {
                    Compression::LZMA1(size)
                } else {
                    Compression::Zlib(size)
                }
            } else {
                Compression::Stored(size)
            }
        } else {
            let compressed_size = actual_crc32.read_u32::<LE>()?;
            let uncompressed_size = actual_crc32.read_u32::<LE>()?;

            let mut stored_size = if compressed_size == u32::MAX {
                Compression::Stored(uncompressed_size)
            } else {
                Compression::Zlib(compressed_size)
            };

            // Add the size of a CRC32 checksum for each 4KiB sub-block
            *stored_size += stored_size.div_ceil(u32::from(INNO_BLOCK_SIZE)) * 4;

            stored_size
        };

        let actual_crc32 = actual_crc32.finalize();
        if actual_crc32 != expected_crc32 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                InnoError::CrcChecksumMismatch {
                    actual: actual_crc32,
                    expected: expected_crc32,
                },
            ));
        }

        Ok(compression)
    }

    fn read_chunk(&mut self) -> Result<bool> {
        let Ok(block_crc32) = self.inner.read_u32::<LE>() else {
            return Ok(false);
        };

        self.length = self.inner.read(&mut self.buffer)?;

        if self.length == 0 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Unexpected Inno block end",
            ));
        }

        let actual_crc32 = crc32fast::hash(&self.buffer[..self.length]);

        if actual_crc32 != block_crc32 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                InnoError::CrcChecksumMismatch {
                    actual: actual_crc32,
                    expected: block_crc32,
                },
            ));
        }

        self.pos = 0;

        Ok(true)
    }
}

impl<R: Read> Read for InnoBlockReader<R> {
    fn read(&mut self, dest: &mut [u8]) -> Result<usize> {
        let mut total_read = 0;

        while total_read < dest.len() {
            if self.pos == self.length && !self.read_chunk()? {
                return Ok(total_read);
            }

            let to_copy = min(dest.len() - total_read, self.length - self.pos);

            dest[total_read..total_read + to_copy]
                .copy_from_slice(&self.buffer[self.pos..self.pos + to_copy]);

            self.pos += to_copy;
            total_read += to_copy;
        }

        Ok(total_read)
    }
}
