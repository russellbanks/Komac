use zerocopy::{FromBytes, Immutable, KnownLayout, LittleEndian, U16, U32};

use super::CoffCharacteristics;

/// In `winnt.h`, it's `IMAGE_FILE_HEADER`. COFF Header.
///
/// Together with the [`Signature`] and the [`OptionalHeader`], it forms the [`IMAGE_NT_HEADERS`].
///
/// ## Position in a modern PE file
///
/// The COFF header is located after the [`Signature`], which in turn is located after the
/// non-standard [`Rich header`], if
/// present, and after the `DosStub`, according to the standard.
///
/// COFF header is followed by the [`OptionalHeader`].
///
/// [`Signature`]: crate::pe::Signature
/// [`OptionalHeader`]: crate::pe::OptionalHeader
/// [`IMAGE_NT_HEADERS`]: https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_nt_headers32
/// [`Rich header`]: https://0xrick.github.io/win-internals/pe3/#rich-header
#[doc(alias("IMAGE_FILE_HEADER"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct CoffHeader {
    #[doc(alias("Machine"))]
    machine: U16<LittleEndian>,

    #[doc(alias("NumberOfSections"))]
    number_of_sections: U16<LittleEndian>,

    #[doc(alias("TimeDateStamp"))]
    time_date_stamp: U32<LittleEndian>,

    #[doc(alias("PointerToSymbolTable"))]
    pointer_to_symbol_table: U32<LittleEndian>,

    #[doc(alias("NumberOfSymbols"))]
    number_of_symbols: U32<LittleEndian>,

    #[doc(alias("SizeOfOptionalHeader"))]
    size_of_optional_header: U16<LittleEndian>,

    #[doc(alias("Characteristics"))]
    characteristics: CoffCharacteristics,
}

impl CoffHeader {
    /// The architecture type of the computer. An image file can only be run
    /// on the specified computer or a system that emulates the specified computer.
    ///
    /// Can be one of the following values:
    ///
    /// * [`COFF_MACHINE_UNKNOWN`],
    /// * [`COFF_MACHINE_ALPHA`],
    /// * [`COFF_MACHINE_ALPHA64`],
    /// * [`COFF_MACHINE_AM33`],
    /// * [`COFF_MACHINE_X86_64`],
    /// * [`COFF_MACHINE_ARM`],
    /// * [`COFF_MACHINE_ARM64`],
    /// * [`COFF_MACHINE_ARMNT`],
    /// * [`COFF_MACHINE_EBC`],
    /// * [`COFF_MACHINE_X86`],
    /// * [`COFF_MACHINE_IA64`],
    /// * [`COFF_MACHINE_LOONGARCH32`],
    /// * [`COFF_MACHINE_LOONGARCH64`],
    /// * [`COFF_MACHINE_M32R`],
    /// * [`COFF_MACHINE_MIPS16`],
    /// * [`COFF_MACHINE_MIPSFPU`],
    /// * [`COFF_MACHINE_MIPSFPU16`],
    /// * [`COFF_MACHINE_POWERPC`],
    /// * [`COFF_MACHINE_POWERPCFP`],
    /// * [`COFF_MACHINE_R4000`],
    /// * [`COFF_MACHINE_RISCV32`],
    /// * [`COFF_MACHINE_RISCV64`],
    /// * [`COFF_MACHINE_RISCV128`],
    /// * [`COFF_MACHINE_SH3`],
    /// * [`COFF_MACHINE_SH3DSP`],
    /// * [`COFF_MACHINE_SH4`],
    /// * [`COFF_MACHINE_SH5`],
    /// * [`COFF_MACHINE_THUMB`],
    /// * [`COFF_MACHINE_WCEMIPSV2`],
    ///
    /// or any other value that is not listed here.
    ///
    /// The constants above are sourced from <https://learn.microsoft.com/windows/win32/debug/pe-format#machine-types>.
    #[inline]
    pub const fn machine(&self) -> u16 {
        self.machine.get()
    }

    /// The number of sections. This indicates the size of the section table, which immediately
    /// follows the headers. Note that the Windows loader limits the number of sections to 96.
    /// [Source](https://learn.microsoft.com/windows/win32/api/winnt/ns-winnt-image_file_header).
    #[inline]
    pub const fn number_of_sections(&self) -> u16 {
        self.number_of_sections.get()
    }

    /// The low 32 bits of the time stamp of the image. This represents the date and time the image
    /// was created by the linker. The value is represented in the number of seconds elapsed since
    /// midnight (00:00:00), January 1, 1970, Universal Coordinated Time, according to the system
    /// clock.
    #[inline]
    pub const fn time_date_stamp(&self) -> u32 {
        self.time_date_stamp.get()
    }

    /// The offset of the symbol table, in bytes, or zero if no COFF symbol table exists.
    ///
    /// Typically, this field is set to 0 because COFF debugging information is deprecated.
    /// [Source](https://0xrick.github.io/win-internals/pe4/#file-header-image_file_header).
    #[inline]
    pub const fn pointer_to_symbol_table(&self) -> u32 {
        self.pointer_to_symbol_table.get()
    }

    /// The number of symbols in the symbol table.
    ///
    /// Typically, this field is set to 0 because COFF debugging information is deprecated.
    /// [Source](https://0xrick.github.io/win-internals/pe4/#file-header-image_file_header).
    #[inline]
    pub const fn number_of_symbols(&self) -> u32 {
        self.number_of_symbols.get()
    }

    /// The size of the optional header, in bytes. This value should be zero for object files.
    ///
    /// The [`OptionalHeader`](crate::pe::OptionalHeader) is meant to represent either the 32-bit or
    /// the 64-bit optional header. The size of the optional header is used to determine which one
    /// it is.
    #[inline]
    pub const fn size_of_optional_header(&self) -> u16 {
        self.size_of_optional_header.get()
    }

    /// The [characteristics] of the image.
    ///
    /// [characteristics]: https://learn.microsoft.com/windows/win32/debug/pe-format#characteristics
    #[inline]
    pub const fn characteristics(&self) -> CoffCharacteristics {
        self.characteristics
    }
}
