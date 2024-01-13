use crate::exe::utils::align;
use crate::exe::vs_header::VSHeader;
use crate::exe::vs_string::VSString;
use color_eyre::eyre::Result;
use std::collections::HashMap;

/// Represents a [`StringTable`](https://docs.microsoft.com/en-us/windows/win32/menurc/stringtable) structure.
pub struct VSStringTable<'data> {
    pub header: VSHeader<'data>,
    pub children: Vec<VSString<'data>>,
}
impl<'data> VSStringTable<'data> {
    /// Parse a `VSStringTable` structure at the given RVA.
    pub fn parse(data: &'data [u8], base_offset: usize) -> Result<Self> {
        let (mut offset, header) = VSHeader::parse(data, base_offset)?;
        let mut consumed = offset - base_offset;
        offset = align(offset, 4);

        let mut children = Vec::<VSString>::new();

        while consumed < *header.length as usize {
            let child = VSString::parse(data, offset)?;

            offset += *child.header.length as usize;
            offset = align(offset, 4);
            consumed = offset - base_offset;
            children.push(child);
        }

        Ok(Self { header, children })
    }

    /// Grab the string table data as a key/value [`HashMap`](HashMap) value.
    pub fn string_map(&self) -> HashMap<String, String> {
        let mut result = HashMap::<String, String>::new();

        for entry in &self.children {
            let entry_key = String::from_utf16_lossy(entry.header.key);
            let entry_value = String::from_utf16_lossy(entry.value);

            result.insert(entry_key, entry_value);
        }

        result
    }
}
