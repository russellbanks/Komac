use zerocopy::{FromBytes, Immutable, KnownLayout, LittleEndian, U16, U32};

#[cfg(doc)]
use super::WindowsFields;

/// Windows specific fields for 32-bit binary (`PE32`). They're also known as "NT additional fields".
///
/// In `winnt.h`, this is a subset of [`IMAGE_OPTIONAL_HEADER32`](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_optional_header32).
///
/// * For 64-bit version, see [`WindowsFields64`].
/// * For unified version, see [`WindowsFields`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct WindowsFields32 {
    /// See docs for [`WindowsFields::image_base`].
    #[doc(alias = "ImageBase")]
    pub(super) image_base: U32<LittleEndian>,

    /// See docs for [`WindowsFields::section_alignment`].
    #[doc(alias = "SectionAlignment")]
    pub(super) section_alignment: U32<LittleEndian>,

    /// See docs for [`WindowsFields::file_alignment`].
    #[doc(alias = "FileAlignment")]
    pub(super) file_alignment: U32<LittleEndian>,

    /// See docs for [`WindowsFields::major_operating_system_version`].
    #[doc(alias = "MajorOperatingSystemVersion")]
    pub(super) major_operating_system_version: U16<LittleEndian>,

    /// See docs for [`WindowsFields::minor_operating_system_version`].
    #[doc(alias = "MinorOperatingSystemVersion")]
    pub(super) minor_operating_system_version: U16<LittleEndian>,

    /// See docs for [`WindowsFields::major_image_version`].
    #[doc(alias = "MajorImageVersion")]
    pub(super) major_image_version: U16<LittleEndian>,

    /// See docs for [`WindowsFields::minor_image_version`].
    #[doc(alias = "MinorImageVersion")]
    pub(super) minor_image_version: U16<LittleEndian>,

    /// See docs for [`WindowsFields::major_subsystem_version`].
    #[doc(alias = "MajorSubsystemVersion")]
    pub(super) major_subsystem_version: U16<LittleEndian>,

    /// See docs for [`WindowsFields::minor_subsystem_version`].
    #[doc(alias = "MinorSubsystemVersion")]
    pub(super) minor_subsystem_version: U16<LittleEndian>,

    /// See docs for [`WindowsFields::win32_version_value`].
    #[doc(alias = "Win32VersionValue")]
    pub(super) win32_version_value: U32<LittleEndian>,

    /// See docs for [`WindowsFields::size_of_image`].
    #[doc(alias = "SizeOfImage")]
    pub(super) size_of_image: U32<LittleEndian>,

    /// See docs for [`WindowsFields::size_of_headers`].
    #[doc(alias = "SizeOfHeaders")]
    pub(super) size_of_headers: U32<LittleEndian>,

    /// See docs for [`WindowsFields::check_sum`].
    #[doc(alias = "CheckSum")]
    pub(super) check_sum: U32<LittleEndian>,

    /// See docs for [`WindowsFields::subsystem`].
    #[doc(alias = "Subsystem")]
    pub(super) subsystem: U16<LittleEndian>,

    /// See docs for [`WindowsFields::dll_characteristics`].
    #[doc(alias = "DllCharacteristics")]
    pub(super) dll_characteristics: U16<LittleEndian>,

    /// See docs for [`WindowsFields::size_of_stack_reserve`].
    #[doc(alias = "SizeOfStackReserve")]
    pub(super) size_of_stack_reserve: U32<LittleEndian>,

    /// See docs for [`WindowsFields::size_of_stack_commit`].
    #[doc(alias = "SizeOfStackCommit")]
    pub(super) size_of_stack_commit: U32<LittleEndian>,

    /// See docs for [`WindowsFields::size_of_heap_reserve`].
    #[doc(alias = "SizeOfHeapReserve")]
    pub(super) size_of_heap_reserve: U32<LittleEndian>,

    /// See docs for [`WindowsFields::size_of_heap_commit`].
    #[doc(alias = "SizeOfHeapCommit")]
    pub(super) size_of_heap_commit: U32<LittleEndian>,

    /// See docs for [`WindowsFields::loader_flags`].
    #[doc(alias = "LoaderFlags")]
    pub(super) loader_flags: U32<LittleEndian>,

    /// See docs for [`WindowsFields::number_of_data_directories`].
    #[doc(alias = "NumberOfRvaAndSizes")]
    pub(super) number_of_data_directories: U32<LittleEndian>,
}

impl WindowsFields32 {
    #[inline]
    pub const fn image_base(&self) -> u32 {
        self.image_base.get()
    }

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
    pub const fn size_of_stack_reserve(&self) -> u32 {
        self.size_of_stack_reserve.get()
    }

    #[inline]
    pub const fn size_of_stack_commit(&self) -> u32 {
        self.size_of_stack_commit.get()
    }

    #[inline]
    pub const fn size_of_heap_reserve(&self) -> u32 {
        self.size_of_heap_reserve.get()
    }

    #[inline]
    pub const fn size_of_heap_commit(&self) -> u32 {
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
