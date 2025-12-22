use std::{io, io::Read, mem::offset_of};

use zerocopy::LE;

use super::DosHeader;
use crate::read::ReadBytesExt;

/// Returns the machine value from a PE executable reader.
///
/// The reader's cursor is expected to be at the start of the executable.
/// This function does not require the underlying reader to implement [`Seek`], instead seeking
/// forward by reading data into the [void].
///
/// [`Seek`]: io::Seek
/// [void]: io::Sink
pub fn machine_from_exe_reader<R>(mut reader: R) -> io::Result<u16>
where
    R: Read,
{
    const PE_POINTER_OFFSET: u8 = offset_of!(DosHeader, pe_pointer) as u8;

    let mut void = io::sink();

    // Seek to COFF header offset inside exe
    io::copy(
        &mut reader.by_ref().take(PE_POINTER_OFFSET.into()),
        &mut void,
    )?;

    let coff_offset = reader.read_u32::<LE>()?;

    let Some(read_count) = coff_offset.checked_sub(PE_POINTER_OFFSET.into()) else {
        return Err(io::Error::other("Invalid PE COFF offset"));
    };

    // Seek to machine value
    io::copy(&mut reader.by_ref().take(u64::from(read_count)), &mut void)?;

    reader.read_u16::<LE>()
}
