use bitflags::bitflags;
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Clone, Copy, Debug, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct CoffCharacteristics(u16);

bitflags! {
    impl CoffCharacteristics: u16 {
        /// Image only, Windows CE, and Microsoft Windows NT and later. This indicates that the file
        /// does not contain base relocations and must therefore be loaded at its preferred base
        /// address. If the base address is not available, the loader reports an error. The default
        /// behavior of the linker is to strip base relocations from executable (EXE) files.
        const IMAGE_FILE_RELOCS_STRIPPED = 1;

        /// Image only. This indicates that the image file is valid and can be run.
        /// If this flag is not set, it indicates a linker error.
        const IMAGE_FILE_EXECUTABLE_IMAGE = 1 << 1;

        /// COFF line numbers have been removed. This flag is deprecated and should be zero.
        const IMAGE_FILE_LINE_NUMS_STRIPPED = 1 << 2;

        /// COFF symbol table entries for local symbols have been removed. This flag is deprecated
        /// and should be zero.
        const IMAGE_FILE_LOCAL_SYMS_STRIPPED = 1 << 3;

        /// Obsolete. Aggressively trim working set. This flag is deprecated for Windows 2000 and
        /// later and must be zero.
        const IMAGE_FILE_AGGRESSIVE_WS_TRIM = 1 << 4;

        /// Application can handle > 2-GB addresses.
        const IMAGE_FILE_LARGE_ADDRESS_AWARE = 1 << 5;

        /// This flag is reserved for future use.
        const RESERVED = 1 << 6;

        /// Little endian: the least significant bit (LSB) precedes the most significant bit (MSB)
        /// in memory.
        /// This flag is deprecated and should be zero.
        const IMAGE_FILE_BYTES_REVERSED_LO = 1 << 7;

        /// Machine is based on a 32-bit-word architecture.
        const IMAGE_FILE_32BIT_MACHINE = 1 << 8;

        /// Debugging information is removed from the image file.
        const IMAGE_FILE_DEBUG_STRIPPED = 1 << 9;

        /// If the image is on removable media, fully load it and copy it to the swap file.
        const IMAGE_FILE_REMOVABLE_RUN_FROM_SWAP = 1 << 10;

        /// If the image is on network media, fully load it and copy it to the swap file.
        const IMAGE_FILE_NET_RUN_FROM_SWAP = 1 << 11;

        /// The image file is a system file, not a user program.
        const IMAGE_FILE_SYSTEM = 1 << 12;

        /// The image file is a dynamic-link library (DLL). Such files are considered executable
        /// files for almost all purposes, although they cannot be directly run.
        const IMAGE_FILE_DLL = 1 << 13;

        /// The file should be run only on a uniprocessor machine.
        const IMAGE_FILE_UP_SYSTEM_ONLY = 1 << 14;

        /// Big endian: the MSB precedes the LSB in memory. This flag is deprecated and should be
        /// zero.
        const IMAGE_FILE_BYTES_REVERSED_HI = 1 << 15;
    }
}
