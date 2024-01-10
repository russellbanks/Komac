use crate::exe::vs_file_flags::VSFileFlags;
use bytemuck::{Pod, Zeroable};

/// Represents a [`VS_FIXEDFILEINFO`](https://docs.microsoft.com/en-us/windows/win32/api/verrsrc/ns-verrsrc-vs_fixedfileinfo) structure.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Eq, PartialEq, Debug)]
pub struct VSFixedFileInfo {
    pub signature: u32,
    pub struct_version: u32,
    pub file_version_ms: u32,
    pub file_version_ls: u32,
    pub product_version_ms: u32,
    pub product_version_ls: u32,
    pub file_flags_mask: u32,
    pub file_flags: VSFileFlags,
    pub file_os: u32,
    pub file_type: u32,
    pub file_subtype: u32,
    pub file_date_ms: u32,
    pub file_date_ls: u32,
}
