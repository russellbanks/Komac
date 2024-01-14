use crate::exe::utils::align;
use crate::exe::vs_fixed_file_info::VSFixedFileInfo;
use crate::exe::vs_header::VSHeader;
use crate::exe::vs_string_file_info::VSStringFileInfo;
use crate::exe::vs_var_file_info::VSVarFileInfo;
use color_eyre::eyre::{bail, OptionExt, Result};
use object::pe::RT_VERSION;
use object::read::pe::{ImageNtHeaders, PeFile};
use object::{LittleEndian, ReadRef};
use std::mem;

/// Represents a [`VS_VERSIONINFO`](https://docs.microsoft.com/en-us/windows/win32/menurc/vs-versioninfo) structure.
pub struct VSVersionInfo<'data> {
    pub header: VSHeader<'data>,
    pub value: Option<&'data VSFixedFileInfo>,
    pub string_file_info: Option<VSStringFileInfo<'data>>,
    pub var_file_info: Option<VSVarFileInfo<'data>>,
}
impl<'data> VSVersionInfo<'data> {
    /// Parse a `VSVersionInfo` structure from the given [`PE`](PE)'s resource directory.
    pub fn parse<'pe_data, Pe, R>(pe: &PeFile<'pe_data, Pe, R>, data: &'data [u8]) -> Result<Self>
    where
        Pe: ImageNtHeaders,
        R: ReadRef<'pe_data>,
    {
        let resource_directory = pe
            .data_directories()
            .resource_directory(pe.data(), &pe.section_table())?
            .ok_or_eyre("No resource directory")?;
        let rt_version = resource_directory
            .root()?
            .entries
            .iter()
            .find(|entry| entry.name_or_id().id() == Some(RT_VERSION))
            .ok_or_eyre("No RT_VERSION was found in the exe")?;
        let manifest_entry = rt_version
            .data(resource_directory)?
            .table()
            .unwrap()
            .entries
            .first()
            .unwrap()
            .data(resource_directory)?
            .table()
            .unwrap()
            .entries
            .first()
            .unwrap()
            .data(resource_directory)?
            .data()
            .unwrap();
        let section = pe
            .section_table()
            .iter()
            .find(|header| header.contains_rva(manifest_entry.offset_to_data.get(LittleEndian)))
            .unwrap();
        // Translate the offset into a usable one
        let base_offset = {
            let mut rva = manifest_entry.offset_to_data.get(LittleEndian);
            rva -= section.virtual_address.get(LittleEndian);
            rva += section.pointer_to_raw_data.get(LittleEndian);
            rva as usize
        };
        let (mut offset, header) = VSHeader::parse(data, base_offset)?;
        let mut consumed = offset;
        offset = align(offset, 4);

        let value;

        if *header.value_length == 0 {
            value = None;
        } else {
            let struct_size = mem::size_of::<VSFixedFileInfo>();
            value = Some(data.read_at(offset as u64).unwrap());
            offset += struct_size;
            consumed = offset - base_offset;
        }

        offset = align(offset, 4);
        let mut string_file_info = None;
        let mut var_file_info = None;

        if consumed < *header.length as usize {
            let (_, header_check) = VSHeader::parse(data, offset)?;

            let header_str = String::from_utf16_lossy(header_check.key);

            if header_str == "StringFileInfo" {
                let string_file_info_tmp = VSStringFileInfo::parse(data, offset)?;

                offset += *string_file_info_tmp.header.length as usize;
                consumed = offset - base_offset;

                string_file_info = Some(string_file_info_tmp);
            } else if header_str == "VarFileInfo" {
                let var_file_info_tmp = VSVarFileInfo::parse(data, offset)?;

                offset += *var_file_info_tmp.header.length as usize;
                consumed = offset - base_offset;

                var_file_info = Some(var_file_info_tmp);
            } else {
                bail!("Unknown VS_VERSIONINFO structure header");
            }
        }

        offset = align(offset, 4);

        if consumed < *header.length as usize {
            let (_, header_check) = VSHeader::parse(data, offset)?;
            let header_str = String::from_utf16_lossy(header_check.key);

            if header_str == "StringFileInfo" {
                let string_file_info_tmp = VSStringFileInfo::parse(data, offset)?;

                string_file_info = Some(string_file_info_tmp);
            } else if header_str == "VarFileInfo" {
                let var_file_info_tmp = VSVarFileInfo::parse(data, offset)?;

                var_file_info = Some(var_file_info_tmp);
            } else {
                bail!("Unknown VS_VERSIONINFO structure header");
            }
        }

        Ok(Self {
            header,
            value,
            string_file_info,
            var_file_info,
        })
    }
}
