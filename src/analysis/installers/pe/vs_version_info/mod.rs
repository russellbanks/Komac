mod var_dword;
mod vs_file_flags;
mod vs_fixed_file_info;
mod vs_header;
mod vs_string;
mod vs_string_file_info;
mod vs_string_table;
mod vs_type;
mod vs_var;
mod vs_var_file_info;

use std::io;

use indexmap::IndexMap;
use var_dword::VarDword;
use vs_file_flags::VSFileFlags;
use vs_fixed_file_info::VSFixedFileInfo;
use vs_header::VSHeader;
use vs_string::VSString;
pub use vs_string_file_info::VSStringFileInfo;
use vs_string_table::VSStringTable;
use vs_type::VSType;
use vs_var::VSVar;
use vs_var_file_info::VSVarFileInfo;
use zerocopy::{FromBytes, TryFromBytes, Unaligned};

/// Represents a [`VS_VERSIONINFO`](https://docs.microsoft.com/windows/win32/menurc/vs-versioninfo) structure.
#[derive(Debug)]
pub struct VSVersionInfo<'a> {
    pub fixed_file_info: &'a VSFixedFileInfo,
    string_file_info: Option<VSStringFileInfo<'a>>,
    var_file_info: Option<VSVarFileInfo<'a>>,
}

impl<'a> VSVersionInfo<'a> {
    const SIGNATURE_LEN: usize = "VS_VERSION_INFO".len() * size_of::<u16>();

    pub fn read_from(data: &'a [u8]) -> io::Result<Self> {
        let header = VSHeader::read_with_key_length_from(data, Some(Self::SIGNATURE_LEN))?;

        if header.key() != "VS_VERSION_INFO" {
            return Err(io::Error::other("Invalid VS_VERSION_INFO signature"));
        }

        let mut offset = header.end_offset;

        let (fixed_file_info, _) = VSFixedFileInfo::try_ref_from_prefix(&data[offset..])
            .map_err(|err| io::Error::other(err.to_string()))?;

        offset += size_of::<VSFixedFileInfo>();
        offset = offset.next_multiple_of(size_of::<u32>());

        let mut string_file_info = None;
        let mut var_file_info = None;

        while offset < usize::from(header.length()) {
            let block_header = VSHeader::read_from(&data[offset..])?;

            let block = &data[offset..offset + usize::from(block_header.length())];

            match block_header.key() {
                "StringFileInfo" => string_file_info = Some(VSStringFileInfo::read_from(block)?),
                "VarFileInfo" => var_file_info = Some(VSVarFileInfo::read_from(block)?),
                _ => {}
            }

            offset += usize::from(block_header.length());
            offset = offset.next_multiple_of(size_of::<u32>());
        }

        Ok(VSVersionInfo {
            fixed_file_info,
            string_file_info,
            var_file_info,
        })
    }

    pub const fn string_file_info(&self) -> Option<&VSStringFileInfo<'a>> {
        self.string_file_info.as_ref()
    }

    pub const fn var_file_info(&self) -> Option<&VSVarFileInfo<'a>> {
        self.var_file_info.as_ref()
    }

    pub fn string_entries(&'a self) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.string_file_info()
            .map(VSStringFileInfo::entries)
            .into_iter()
            .flatten()
    }

    pub fn string_table(&self) -> IndexMap<&str, &str> {
        self.string_file_info()
            .map(VSStringFileInfo::table)
            .unwrap_or_default()
    }
}
