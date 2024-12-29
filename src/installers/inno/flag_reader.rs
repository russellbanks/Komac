use byteorder::ReadBytesExt;
use std::io::{Read, Result};
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

    pub fn add<I: IntoIterator<Item = E>>(&mut self, flags: I) -> Result<()> {
        for flag in flags {
            if self.pos == 0 {
                self.bytes += 1;
                self.value = self.reader.read_u8()?;
            }

            if self.value & (1 << self.pos) != 0 {
                self.flags |= flag;
            }

            self.pos = (self.pos + 1) % u8::BITS as usize;
        }

        Ok(())
    }

    pub fn finalize(self) -> Result<E> {
        if self.bytes == 3 {
            self.reader.read_u8()?;
        }
        Ok(self.flags)
    }
}

pub mod read_flags {
    macro_rules! read_flags {
        ($reader_init:expr, $(,)?) => {{
            let mut flag_reader = crate::installers::inno::flag_reader::FlagReader::new($reader_init);
            flag_reader.finalize()
        }};

        ($reader_init:expr, [$($flags:expr),+ $(,)?]) => {{
            let mut flag_reader = crate::installers::inno::flag_reader::FlagReader::new($reader_init);
            flag_reader.add([$($flags),+])?;
            flag_reader.finalize()
        }};

        ($reader_init:expr, [$($flags:expr),+ $(,)?], $($rest:tt)*) => {{
            let mut flag_reader = crate::installers::inno::flag_reader::FlagReader::new($reader_init);
            flag_reader.add([$($flags),+])?;
            crate::installers::inno::flag_reader::read_flags::read_flags_internal!(flag_reader, $($rest)*)
        }};

        ($reader_init:expr, if $cond:expr => $flag:expr) => {{
            let mut flag_reader = crate::installers::inno::flag_reader::FlagReader::new($reader_init);
            if $cond {
                flag_reader.add($flag)?;
            }
            flag_reader.finalize()
        }};

        ($reader_init:expr, if $cond:expr => $flag:expr, $($rest:tt)*) => {{
            let mut flag_reader = crate::installers::inno::flag_reader::FlagReader::new($reader_init);
            if $cond {
                flag_reader.add($flag)?;
            }
            crate::installers::inno::flag_reader::read_flags::read_flags_internal!(flag_reader, $($rest)*)
        }};

        ($reader_init:expr, $flag:expr) => {{
            let mut flag_reader = crate::installers::inno::flag_reader::FlagReader::new($reader_init);
            flag_reader.add($flag)?;
            flag_reader.finalize()
        }};

        ($reader_init:expr, $flag:expr, $($rest:tt)*) => {{
            let mut flag_reader = crate::installers::inno::flag_reader::FlagReader::new($reader_init);
            flag_reader.add($flag)?;
            crate::installers::inno::flag_reader::read_flags::read_flags_internal!(flag_reader, $($rest)*)
        }};
    }

    macro_rules! read_flags_internal {
        ($reader:expr) => {
            $reader.finalize()
        };

        ($reader:expr, [$($flags:expr),+ $(,)?]) => {{
            $reader.add([$($flags),+])?;
            $reader.finalize()
        }};

        ($reader:expr, [$($flags:expr),+ $(,)?], $($rest:tt)*) => {{
            $reader.add([$($flags),+])?;
            crate::installers::inno::flag_reader::read_flags::read_flags_internal!($reader, $($rest)*)
        }};

        ($reader:expr, if $cond:expr => $flag:expr) => {{
            if $cond {
                $reader.add($flag)?;
            }
            $reader.finalize()
        }};

        ($reader:expr, if $cond:expr => $flag:expr, $($rest:tt)*) => {{
            if $cond {
                $reader.add($flag)?;
            }
            crate::installers::inno::flag_reader::read_flags::read_flags_internal!($reader, $($rest)*)
        }};

        ($reader:expr, $flag:expr) => {{
            $reader.add($flag)?;
            $reader.finalize()
        }};

        ($reader:expr, $flag:expr, $($rest:tt)*) => {{
            $reader.add($flag)?;
            crate::installers::inno::flag_reader::read_flags::read_flags_internal!($reader, $($rest)*)
        }};
    }

    pub(crate) use read_flags;
    pub(crate) use read_flags_internal;
}
