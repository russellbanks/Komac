pub mod burn;
mod exe;
pub mod inno;
mod msi;
pub mod msix_family;
pub mod nsis;
pub mod pe;
pub mod utils;
mod zip;

use std::{io, io::SeekFrom};

pub use burn::Burn;
pub use exe::Exe;
pub use msi::Msi;
pub use nsis::Nsis;
pub use zip::Zip;

use crate::{
    analysis::installers::pe::{CoffHeader, DosHeader, OptionalHeader, SectionTable, Signature},
    read::ReadBytesExt,
};

pub struct PE {
    pub dos_header: DosHeader,
    pub coff_header: CoffHeader,
    pub optional_header: OptionalHeader,
    pub section_table: SectionTable,
}

impl PE {
    pub fn read_from<R>(mut reader: R) -> io::Result<Self>
    where
        R: io::Read + io::Seek,
    {
        reader.seek(SeekFrom::Start(0))?;
        // Read DOS Header
        let dos_header = DosHeader::try_read_from_io(&mut reader)?;

        // Seek to PE header
        reader.seek(SeekFrom::Start(dos_header.pe_pointer().into()))?;

        // Read PE Signature
        let _signature = Signature::try_read_from(&mut reader)?; // PE/0/0

        // Read COFF header
        let coff_header = reader.read_t::<CoffHeader>()?;

        // Read optional header
        let optional_header = OptionalHeader::read_from(&mut reader)?;

        // Read the section table
        let section_table = SectionTable::read_from(&mut reader, coff_header)?;

        Ok(Self {
            dos_header,
            coff_header,
            optional_header,
            section_table,
        })
    }

    #[inline]
    pub const fn machine(&self) -> u16 {
        self.coff_header.machine()
    }
}

pub const IMAGE_FILE_MACHINE_UNKNOWN: u16 = 0;
/// Useful for indicating we want to interact with the host and not a WoW guest.
pub const IMAGE_FILE_MACHINE_TARGET_HOST: u16 = 0x0001;
/// Intel 386.
pub const IMAGE_FILE_MACHINE_I386: u16 = 0x014c;
/// MIPS little-endian, 0x160 big-endian
pub const IMAGE_FILE_MACHINE_R3000: u16 = 0x0162;
/// MIPS little-endian
pub const IMAGE_FILE_MACHINE_R4000: u16 = 0x0166;
/// MIPS little-endian
pub const IMAGE_FILE_MACHINE_R10000: u16 = 0x0168;
/// MIPS little-endian WCE v2
pub const IMAGE_FILE_MACHINE_WCEMIPSV2: u16 = 0x0169;
/// Alpha_AXP
pub const IMAGE_FILE_MACHINE_ALPHA: u16 = 0x0184;
/// SH3 little-endian
pub const IMAGE_FILE_MACHINE_SH3: u16 = 0x01a2;
pub const IMAGE_FILE_MACHINE_SH3DSP: u16 = 0x01a3;
/// SH3E little-endian
pub const IMAGE_FILE_MACHINE_SH3E: u16 = 0x01a4;
/// SH4 little-endian
pub const IMAGE_FILE_MACHINE_SH4: u16 = 0x01a6;
/// SH5
pub const IMAGE_FILE_MACHINE_SH5: u16 = 0x01a8;
/// ARM Little-Endian
pub const IMAGE_FILE_MACHINE_ARM: u16 = 0x01c0;
/// ARM Thumb/Thumb-2 Little-Endian
pub const IMAGE_FILE_MACHINE_THUMB: u16 = 0x01c2;
/// ARM Thumb-2 Little-Endian
pub const IMAGE_FILE_MACHINE_ARMNT: u16 = 0x01c4;
pub const IMAGE_FILE_MACHINE_AM33: u16 = 0x01d3;
/// IBM PowerPC Little-Endian
pub const IMAGE_FILE_MACHINE_POWERPC: u16 = 0x01F0;
pub const IMAGE_FILE_MACHINE_POWERPCFP: u16 = 0x01f1;
/// IBM PowerPC Big-Endian
pub const IMAGE_FILE_MACHINE_POWERPCBE: u16 = 0x01f2;
/// Intel 64
pub const IMAGE_FILE_MACHINE_IA64: u16 = 0x0200;
/// MIPS
pub const IMAGE_FILE_MACHINE_MIPS16: u16 = 0x0266;
/// ALPHA64
pub const IMAGE_FILE_MACHINE_ALPHA64: u16 = 0x0284;
/// MIPS
pub const IMAGE_FILE_MACHINE_MIPSFPU: u16 = 0x0366;
/// MIPS
pub const IMAGE_FILE_MACHINE_MIPSFPU16: u16 = 0x0466;
pub const IMAGE_FILE_MACHINE_AXP64: u16 = IMAGE_FILE_MACHINE_ALPHA64;
/// Infineon
pub const IMAGE_FILE_MACHINE_TRICORE: u16 = 0x0520;
pub const IMAGE_FILE_MACHINE_CEF: u16 = 0x0CEF;
/// EFI Byte Code
pub const IMAGE_FILE_MACHINE_EBC: u16 = 0x0EBC;
/// AMD64 (K8)
pub const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
/// M32R little-endian
pub const IMAGE_FILE_MACHINE_M32R: u16 = 0x9041;
/// ARM64 Little-Endian
pub const IMAGE_FILE_MACHINE_ARM64: u16 = 0xAA64;
/// ARM64EC ("Emulation Compatible")
pub const IMAGE_FILE_MACHINE_ARM64EC: u16 = 0xA641;
pub const IMAGE_FILE_MACHINE_CEE: u16 = 0xC0EE;
/// RISCV32
pub const IMAGE_FILE_MACHINE_RISCV32: u16 = 0x5032;
/// RISCV64
pub const IMAGE_FILE_MACHINE_RISCV64: u16 = 0x5064;
/// RISCV128
pub const IMAGE_FILE_MACHINE_RISCV128: u16 = 0x5128;
/// ARM64X (Mixed ARM64 and ARM64EC)
pub const IMAGE_FILE_MACHINE_ARM64X: u16 = 0xA64E;
/// CHPE x86 ("Compiled Hybrid Portable Executable")
pub const IMAGE_FILE_MACHINE_CHPE_X86: u16 = 0x3A64;
