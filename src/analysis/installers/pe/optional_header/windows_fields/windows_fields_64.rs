use zerocopy::{FromBytes, Immutable, KnownLayout, LittleEndian, U16, U32, U64};

#[cfg(doc)]
use super::WindowsFields;
use super::WindowsFields32;

/// Windows specific fields for 64-bit binary (`PE32+`). They're also known as "NT additional fields".
///
/// In `winnt.h`, this is a subset of [`IMAGE_OPTIONAL_HEADER64`](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_optional_header64).
///
/// *Note: at the moment of writing, [`WindowsFields`] is an alias for `WindowsFields64`. Though [nominally equivalent](https://en.wikipedia.org/wiki/Nominal_type_system),
/// they're semantically distinct.*
///
/// * For 32-bit version, see [`WindowsFields32`].
/// * For unified version, see [`WindowsFields`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct WindowsFields64 {
    /// See docs for [`WindowsFields::image_base`].
    #[doc(alias = "ImageBase")]
    image_base: U64<LittleEndian>,

    /// See docs for [`WindowsFields::file_alignment`].
    #[doc(alias = "SectionAlignment")]
    section_alignment: U32<LittleEndian>,
    /// The alignment factor (in bytes) that is used to align the raw data of sections in the image file.
    ///
    /// The value should be a power of 2 between 512 and 64 K, inclusive.
    ///
    /// If the [`section_alignment`](WindowsFields64::section_alignment) is less than the architecture's page size,
    /// then [`file_alignment`](WindowsFields64::file_alignment) must match [`section_alignment`](WindowsFields64::section_alignment).
    ///
    /// If [`file_alignment`](WindowsFields64::file_alignment) is less than [`section_alignment`](WindowsFields64::section_alignment),
    /// then remainder will be padded with zeroes in order to maintain the alignment boundaries.
    /// [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/nt-headers/optional-header/).
    ///
    /// The default value is 512.
    #[doc(alias = "FileAlignment")]
    file_alignment: U32<LittleEndian>,
    /// The major version number of the required operating system.
    #[doc(alias = "MajorOperatingSystemVersion")]
    major_operating_system_version: U16<LittleEndian>,
    /// The minor version number of the required operating system.
    #[doc(alias = "MinorOperatingSystemVersion")]
    minor_operating_system_version: U16<LittleEndian>,
    /// The major version number of the image.
    #[doc(alias = "MajorImageVersion")]
    major_image_version: U16<LittleEndian>,
    /// The minor version number of the image.
    #[doc(alias = "MinorImageVersion")]
    minor_image_version: U16<LittleEndian>,
    /// The major version number of the subsystem.
    #[doc(alias = "MajorSubsystemVersion")]
    major_subsystem_version: U16<LittleEndian>,
    /// The minor version number of the subsystem.
    #[doc(alias = "MinorSubsystemVersion")]
    minor_subsystem_version: U16<LittleEndian>,
    /// Reserved, must be zero.
    #[doc(alias = "Win32VersionValue")]
    win32_version_value: U32<LittleEndian>,
    /// The size (in bytes) of the image, including all headers, as the image is loaded in memory.
    ///
    /// It must be a multiple of the [`section_alignment`](WindowsFields64::section_alignment).
    #[doc(alias = "SizeOfImage")]
    size_of_image: U32<LittleEndian>,
    /// The combined size of an MS-DOS stub, PE header, and section headers rounded up to a multiple of
    /// [`file_alignment`](WindowsFields64::file_alignment).
    #[doc(alias = "SizeOfHeaders")]
    size_of_headers: U32<LittleEndian>,
    /// The image file checksum. The algorithm for computing the checksum is incorporated into IMAGHELP.DLL.
    ///
    /// The following are checked for validation at load time:
    /// * all drivers,
    /// * any DLL loaded at boot time, and
    /// * any DLL that is loaded into a critical Windows process.
    #[doc(alias = "CheckSum")]
    check_sum: U32<LittleEndian>,
    /// The subsystem that is required to run this image.
    ///
    /// The subsystem can be one of the values in the [`goblin::pe::subsystem`](crate::pe::subsystem) module.
    #[doc(alias = "Subsystem")]
    subsystem: U16<LittleEndian>,
    /// DLL characteristics of the image.
    ///
    /// DLL characteristics can be one of the values in the
    /// [`goblin::pe::dll_characteristic`](crate::pe::dll_characteristic) module.
    #[doc(alias = "DllCharacteristics")]
    dll_characteristics: U16<LittleEndian>,
    /// The size of the stack to reserve. Only [`WindowsFields::size_of_stack_commit`] is committed;
    /// the rest is made available one page at a time until the reserve size is reached.
    ///
    /// In the context of memory management in operating systems, "commit" refers to the act of allocating physical memory
    /// to back a portion of the virtual memory space.
    ///
    /// When a program requests memory, the operating system typically allocates virtual memory space for it. However,
    /// this virtual memory space doesn't immediately consume physical memory (RAM) resources. Instead, physical memory
    /// is only allocated when the program actually uses (or accesses) that portion of the virtual memory space.
    /// This allocation of physical memory to back virtual memory is called "committing" memory.
    #[doc(alias = "SizeOfStackReserve")]
    size_of_stack_reserve: U64<LittleEndian>,
    /// The size of the stack to commit.
    #[doc(alias = "SizeOfStackCommit")]
    size_of_stack_commit: U64<LittleEndian>,
    ///  The size of the local heap space to reserve. Only [`WindowsFields::size_of_heap_commit`] is committed; the rest
    /// is made available one page at a time until the reserve size is reached.
    #[doc(alias = "SizeOfHeapReserve")]
    size_of_heap_reserve: U64<LittleEndian>,
    /// The size of the local heap space to commit.
    #[doc(alias = "SizeOfHeapCommit")]
    size_of_heap_commit: U64<LittleEndian>,
    /// Reserved, must be zero.
    #[doc(alias = "LoaderFlags")]
    loader_flags: U32<LittleEndian>,
    /// The number of data-directory entries in the remainder of the optional header. Each describes a location and size.
    #[doc(alias = "NumberOfRvaAndSizes")]
    number_of_data_directories: U32<LittleEndian>,
}

impl WindowsFields64 {
    /// The *preferred* yet rarely provided address of the first byte of image when loaded into memory; must be a
    /// multiple of 64 K.
    ///
    /// This address is rarely used because Windows uses memory protection mechanisms like Address Space Layout
    /// Randomization (ASLR). As a result, it’s rare to see an image mapped to the preferred address. Instead,
    /// the Windows PE Loader maps the file to a different address with an unused memory range. This process
    /// would create issues because some addresses that would have been constant are now changed. The Loader
    /// addresses this via a process called PE relocation which fixes these constant addresses to work with the
    /// new image base. The relocation section (.reloc) holds data essential to this relocation process.
    /// [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/nt-headers/optional-header/).
    ///
    /// * The default address for DLLs is 0x10000000.
    /// * The default for Windows CE EXEs is 0x00010000.
    /// * The default for Windows NT, Windows 2000, Windows XP, Windows 95, Windows 98, and Windows Me is 0x00400000.
    ///
    /// ## Position in PE binary
    ///
    /// Windows fields are located inside [`OptionalHeader`] after [`StandardFields`] and before the
    /// [`DataDirectories`](data_directories::DataDirectories).
    ///
    /// ## Related structures
    ///
    /// * For 32-bit version, see [`WindowsFields32`].
    /// * For unified version, see [`WindowsFields`], especially the note on nominal equivalence.
    #[inline]
    pub const fn image_base(&self) -> u64 {
        self.image_base.get()
    }

    /// Holds a byte value used for section alignment in memory.
    ///
    /// This value must be greater than or equal to
    /// [`file_alignment`](WindowsFields64::file_alignment), which is the next field.
    ///
    /// When loaded into memory, sections are aligned in memory boundaries that are multiples of this value.
    ///
    /// If the value is less than the architecture’s page size, then the value should match
    /// [`file_alignment`](WindowsFields64::file_alignment).
    /// [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/nt-headers/optional-header/).
    ///
    /// The default value is the page size for the architecture.
    #[inline]
    pub const fn section_alignment(&self) -> u32 {
        self.section_alignment.get()
    }

    #[inline]
    pub const fn file_alignment(&self) -> u32 {
        self.file_alignment.get()
    }

    #[inline]
    pub const fn major_operating_system_version(&self) -> u16 {
        self.major_operating_system_version.get()
    }

    #[inline]
    pub const fn minor_operating_system_version(&self) -> u16 {
        self.minor_operating_system_version.get()
    }

    #[inline]
    pub const fn major_image_version(&self) -> u16 {
        self.major_image_version.get()
    }

    #[inline]
    pub const fn minor_image_version(&self) -> u16 {
        self.minor_image_version.get()
    }

    #[inline]
    pub const fn major_subsystem_version(&self) -> u16 {
        self.major_subsystem_version.get()
    }

    #[inline]
    pub const fn minor_subsystem_version(&self) -> u16 {
        self.minor_subsystem_version.get()
    }

    #[inline]
    pub const fn win32_version_value(&self) -> u32 {
        self.win32_version_value.get()
    }

    #[inline]
    pub const fn size_of_image(&self) -> u32 {
        self.size_of_image.get()
    }

    #[inline]
    pub const fn size_of_headers(&self) -> u32 {
        self.size_of_headers.get()
    }

    #[inline]
    pub const fn check_sum(&self) -> u32 {
        self.check_sum.get()
    }

    #[inline]
    pub const fn subsystem(&self) -> u16 {
        self.subsystem.get()
    }

    #[inline]
    pub const fn dll_characteristics(&self) -> u16 {
        self.dll_characteristics.get()
    }

    #[inline]
    pub const fn size_of_stack_reserve(&self) -> u64 {
        self.size_of_stack_reserve.get()
    }

    #[inline]
    pub const fn size_of_stack_commit(&self) -> u64 {
        self.size_of_stack_commit.get()
    }

    #[inline]
    pub const fn size_of_heap_reserve(&self) -> u64 {
        self.size_of_heap_reserve.get()
    }

    #[inline]
    pub const fn size_of_heap_commit(&self) -> u64 {
        self.size_of_heap_commit.get()
    }

    #[inline]
    pub const fn loader_flags(&self) -> u32 {
        self.loader_flags.get()
    }

    #[inline]
    pub const fn number_of_data_directories(&self) -> u32 {
        self.number_of_data_directories.get()
    }
}

impl From<WindowsFields32> for WindowsFields64 {
    fn from(windows: WindowsFields32) -> Self {
        Self {
            image_base: windows.image_base.into(),
            section_alignment: windows.section_alignment,
            file_alignment: windows.file_alignment,
            major_operating_system_version: windows.major_operating_system_version,
            minor_operating_system_version: windows.minor_operating_system_version,
            major_image_version: windows.major_image_version,
            minor_image_version: windows.minor_image_version,
            major_subsystem_version: windows.major_subsystem_version,
            minor_subsystem_version: windows.minor_subsystem_version,
            win32_version_value: windows.win32_version_value,
            size_of_image: windows.size_of_image,
            size_of_headers: windows.size_of_headers,
            check_sum: windows.check_sum,
            subsystem: windows.subsystem,
            dll_characteristics: windows.dll_characteristics,
            size_of_stack_reserve: windows.size_of_stack_reserve.into(),
            size_of_stack_commit: windows.size_of_stack_commit.into(),
            size_of_heap_reserve: windows.size_of_heap_reserve.into(),
            size_of_heap_commit: windows.size_of_heap_commit.into(),
            loader_flags: windows.loader_flags,
            number_of_data_directories: windows.number_of_data_directories,
        }
    }
}
