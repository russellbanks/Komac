#![expect(unused)]

mod coff;
pub mod dos;
pub mod optional_header;
pub mod resource;
mod section_table;
mod signature;

use std::{
    io,
    io::{Error, Read, Seek, SeekFrom},
};

pub use coff::CoffHeader;
pub use dos::DosHeader;
pub use optional_header::OptionalHeader;
pub use section_table::{SectionHeader, SectionTable};
pub use signature::Signature;

use crate::{
    analysis::installers::pe::{
        optional_header::DataDirectory,
        resource::{ImageResourceDataEntry, ResourceDirectory, SectionReader},
    },
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
        R: Read + Seek,
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

    pub fn find_section(&self, name: [u8; 8]) -> Option<&SectionHeader> {
        self.section_table
            .sections()
            .iter()
            .find(|section| section.raw_name() == name)
    }

    /// Returns the maximum file offset occupied by any section.
    ///
    /// This method iterates over all sections in the section table and computes, for each section,
    /// the end offset in the underlying file by adding the section’s `pointer_to_raw_data`
    /// (its starting file offset) to its `size_of_raw_data` (its length in bytes).
    ///
    /// The maximum of these computed end offsets is returned, representing the highest byte offset
    /// covered by any section.
    ///
    /// # Returns
    ///
    /// * `Some(u64)` containing the largest section end offset if at least one
    ///   section is present.
    /// * `None` if the section table contains no sections.
    pub fn overlay_offset(&self) -> Option<u64> {
        self.section_table
            .sections()
            .iter()
            .map(|section| {
                u64::from(section.pointer_to_raw_data()) + u64::from(section.size_of_raw_data())
            })
            .max()
    }

    #[inline]
    pub fn resource_table(&self) -> Option<&DataDirectory> {
        self.optional_header.data_directories.resource_table()
    }

    pub fn manifest<R>(&self, mut reader: R) -> io::Result<String>
    where
        R: Read + Seek,
    {
        let resource_table = self
            .resource_table()
            .ok_or_else(|| Error::other("No PE resource table found"))?;

        // Get the actual file offset of the resource directory section
        let resource_directory_offset = resource_table.file_offset(&self.section_table)?;

        let section_reader = SectionReader::new(
            &mut reader,
            resource_directory_offset.into(),
            resource_table.size().into(),
        )?;

        let mut resource_directory = ResourceDirectory::new(section_reader)?;

        let _manifest = resource_directory.find_manifest()?;

        let manifest_entry = resource_directory
            .current_directory_table()
            .entries()
            .next()
            .ok_or_else(|| Error::other("No manifest entry found"))?;

        let manifest_directory_table = manifest_entry
            .data(&mut resource_directory)?
            .table()
            .ok_or_else(|| Error::other("No manifest directory table found"))?;

        let manifest_directory = manifest_directory_table
            .entries()
            .next()
            .ok_or_else(|| Error::other("No manifest directory found"))?;

        let manifest_data_entry_offset = manifest_directory.file_offset(resource_directory_offset);
        reader.seek(SeekFrom::Start(manifest_data_entry_offset.into()))?;

        let manifest_data_entry = reader.read_t::<ImageResourceDataEntry>()?;
        let manifest_offset = self
            .section_table
            .to_file_offset(manifest_data_entry.offset_to_data())?;

        reader.seek(SeekFrom::Start(manifest_offset.into()))?;
        let mut manifest_reader = reader.take(manifest_data_entry.size().into());

        let mut manifest = String::with_capacity(manifest_data_entry.size() as usize);
        manifest_reader.read_to_string(&mut manifest)?;

        Ok(manifest)
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
