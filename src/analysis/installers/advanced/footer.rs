use std::{
    fmt,
    io::{Read, Seek, SeekFrom},
    mem::offset_of,
};

use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U32, Unaligned};

use super::AdvancedInstallerError;
use crate::read::ReadBytesExt;

/// Advanced Installer Footer.
///
/// Sources:
/// * <https://github.com/SabreTools/SabreTools.Serialization/blob/main/SabreTools.Data.Models/AdvancedInstaller/Footer.cs>
/// * <https://gist.github.com/KasparNagu/9ee02cb62d81d9e4c7a833518a710d6e>
/// * <https://gist.github.com/cw2k/2b2163c422183b884b7405bc0e09dfb2>
#[derive(Clone, Copy, FromBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct Footer {
    reserved: U32<LE>,

    /// Absolute offset of the footer within the EXE.
    offset: U32<LE>,

    num_files: U32<LE>,

    /// 100
    version: U32<LE>,

    info_offset: U32<LE>,

    table_pointer: U32<LE>,

    file_data_start: U32<LE>,

    hex_string: [u8; 32],

    unknown: U32<LE>,

    /// "ADVINSTSFX"
    signature: [u8; 10],
}

impl Footer {
    pub const SIGNATURE: &[u8; 10] = b"ADVINSTSFX";

    pub const SIGNATURE_OFFSET: usize = offset_of!(Footer, signature);

    pub fn find<R: Read + Seek>(reader: &mut R) -> Result<Self, AdvancedInstallerError> {
        const SEARCH_BLOCK_SIZE: usize = 16 * 1024;

        let search_start = reader
            .seek(SeekFrom::End(-(SEARCH_BLOCK_SIZE as i64)))
            .map_err(|_| AdvancedInstallerError::NotAdvancedInstallerFile)?;

        let mut buf = [0; SEARCH_BLOCK_SIZE];
        reader.read_exact(&mut buf)?;

        let signature_pos = buf
            .array_windows()
            .rposition(|window| window == Footer::SIGNATURE)
            .ok_or(AdvancedInstallerError::NotAdvancedInstallerFile)?;

        let footer_offset = search_start + signature_pos as u64 - Footer::SIGNATURE_OFFSET as u64;
        reader.seek(SeekFrom::Start(footer_offset))?;

        reader.read_t::<Self>().map_err(AdvancedInstallerError::Io)
    }

    /// Returns the absolute offset of the footer within the EXE.
    #[inline]
    pub const fn offset(&self) -> u32 {
        self.offset.get()
    }

    /// Returns the number of files in the installer.
    #[inline]
    pub const fn num_files(&self) -> u32 {
        self.num_files.get()
    }

    /// Returns the Advanced Installer version.
    #[inline]
    pub const fn version(&self) -> u32 {
        self.version.get()
    }

    #[inline]
    pub const fn info_offset(&self) -> u32 {
        self.info_offset.get()
    }

    #[inline]
    pub const fn table_pointer(&self) -> u32 {
        self.table_pointer.get()
    }

    #[inline]
    pub const fn file_data_start(&self) -> u32 {
        self.file_data_start.get()
    }
}

impl fmt::Debug for Footer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Footer")
            .field("unknown", &self.reserved.get())
            .field("offset", &self.offset())
            .field("num_files", &self.num_files())
            .field("version", &self.version())
            .field("info_offset", &self.info_offset())
            .field("table_pointer", &self.table_pointer())
            .field("file_data_start", &self.file_data_start())
            .field("hex_string", &String::from_utf8_lossy(&self.hex_string))
            .field("unknown", &self.unknown.get())
            .field("signature", &String::from_utf8_lossy(&self.signature))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use super::Footer;

    #[test]
    fn size() {
        assert_eq!(size_of::<Footer>(), 74);
    }

    #[test]
    fn alignment() {
        assert_eq!(align_of::<Footer>(), 1);
    }

    #[test]
    fn signature_offset() {
        assert_eq!(Footer::SIGNATURE_OFFSET, 64);
    }
}
