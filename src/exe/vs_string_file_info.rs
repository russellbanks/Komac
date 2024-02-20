use crate::exe::utils::align;
use crate::exe::vs_header::VSHeader;
use crate::exe::vs_string_table::VSStringTable;
use color_eyre::eyre::Result;
use object::ReadRef;

/// Represents a [`StringFileInfo`](https://docs.microsoft.com/en-us/windows/win32/menurc/stringfileinfo) structure.
pub struct VSStringFileInfo<'data> {
    pub header: VSHeader<'data>,
    pub children: Vec<VSStringTable<'data>>,
}
impl<'data> VSStringFileInfo<'data> {
    /// Parse a `VSStringFileInfo` structure at the given [`RVA`](RVA).
    pub fn parse<R: ReadRef<'data>>(data: R, base_offset: u64) -> Result<Self> {
        let (mut offset, header) = VSHeader::parse(data, base_offset)?;
        let mut consumed = offset - base_offset;
        offset = align(offset, 4);

        let mut children = Vec::<VSStringTable>::new();

        while consumed < u64::from(*header.length) {
            let child = VSStringTable::parse(data, offset)?;

            offset += u64::from(*child.header.length);
            offset = align(offset, 4);
            consumed = offset - base_offset;
            children.push(child);
        }

        Ok(Self { header, children })
    }
}
