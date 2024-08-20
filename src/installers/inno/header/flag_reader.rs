use byteorder::ReadBytesExt;
use std::io;
use std::io::Read;
use std::ops::BitOrAssign;

/// Represents a reader for loading a flag set where the possible flags are not known at compile-time.
/// The flags are stored as packed bitfields, with 1 byte for every 8 flags.
/// 3-byte bitfields are padded to 4 bytes for non-16-bit builds.
pub struct FlagReader<'reader, E, R> {
    reader: &'reader mut R,
    flags: E,
    pos: usize,
    value: u8,
    bytes: usize,
}

impl<'reader, E, R> FlagReader<'reader, E, R>
where
    E: BitOrAssign + Default,
    R: Read,
{
    pub fn new(reader: &'reader mut R) -> Self {
        FlagReader {
            reader,
            flags: E::default(),
            pos: 0,
            value: 0,
            bytes: 0,
        }
    }

    pub fn add(&mut self, flag: E) -> io::Result<()> {
        if self.pos == 0 {
            self.bytes += 1;
            self.value = self.reader.read_u8()?;
        }

        if self.value & (1 << self.pos) != 0 {
            self.flags |= flag;
        }

        self.pos = (self.pos + 1) % u8::BITS as usize;
        Ok(())
    }

    pub fn finalize(self) -> io::Result<E> {
        if self.bytes == 3 {
            self.reader.read_u8()?;
        }
        Ok(self.flags)
    }
}
