use std::{fmt, mem::offset_of};

use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U32};

/// Advanced Installer Footer.
///
/// Sources:
/// * <https://github.com/SabreTools/SabreTools.Serialization/blob/main/SabreTools.Data.Models/AdvancedInstaller/Footer.cs>
/// * <https://gist.github.com/KasparNagu/9ee02cb62d81d9e4c7a833518a710d6e>
/// * <https://gist.github.com/cw2k/2b2163c422183b884b7405bc0e09dfb2>
#[derive(Clone, Copy, FromBytes, KnownLayout, Immutable)]
#[repr(C, align(4))]
pub struct Footer {
    /// Unknown
    ///
    /// Observed to be always 0.
    unknown: U32<LE>,

    /// Absolute offset of the footer within the EXE.
    offset: U32<LE>,

    num_files: U32<LE>,
    version: U32<LE>,
    info_offset: U32<LE>,
    table_pointer: U32<LE>,
    file_data_start: U32<LE>,
    hex_string: [u8; 32],
    unknown1: U32<LE>,

    /// "ADVINSTSFX\0"
    signature: [u8; 11],
}

impl Footer {
    pub const SIGNATURE: &[u8; 11] = b"ADVINSTSFX\0";

    pub const SIGNATURE_OFFSET: usize = offset_of!(Footer, signature);

    #[inline]
    pub const fn num_files(&self) -> u32 {
        self.num_files.get()
    }

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
            .field("unknown", &self.unknown.get())
            .field("offset", &self.offset.get())
            .field("num_files", &self.num_files())
            .field("version", &self.version())
            .field("info_offset", &self.info_offset())
            .field("table_pointer", &self.table_pointer())
            .field("file_data_start", &self.file_data_start())
            .field("hex_string", &String::from_utf8_lossy(&self.hex_string))
            .field("unknown", &self.unknown1.get())
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
        assert_eq!(size_of::<Footer>(), 76);
    }

    #[test]
    fn alignment() {
        assert_eq!(align_of::<Footer>(), align_of::<u32>());
    }
}
