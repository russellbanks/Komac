use crate::exe::utils::align;
use crate::exe::vs_header::VSHeader;
use crate::exe::vs_var::VSVar;
use color_eyre::eyre::Result;
use object::ReadRef;

/// Represents a [`VarFileInfo`](https://docs.microsoft.com/en-us/windows/win32/menurc/varfileinfo) structure.
pub struct VSVarFileInfo<'data> {
    pub header: VSHeader<'data>,
    pub children: Vec<VSVar<'data>>,
}
impl<'data> VSVarFileInfo<'data> {
    /// Parse a `VSVarFileInfo` structure at the given [`RVA`](RVA).
    pub fn parse<R: ReadRef<'data>>(data: R, base_offset: u64) -> Result<Self> {
        let (mut offset, header) = VSHeader::parse(data, base_offset)?;
        let mut consumed = offset;
        offset = align(offset, 4);

        let mut children = Vec::<VSVar>::new();

        while consumed < u64::from(*header.length) {
            let child = VSVar::parse(data, offset)?;

            offset += u64::from(*child.header.length);
            offset = align(offset, 4);
            consumed = offset - base_offset;
            children.push(child);
        }

        Ok(Self { header, children })
    }
}
