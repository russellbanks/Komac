use crate::exe::utils::align;
use crate::exe::var_dword::VarDword;
use crate::exe::vs_header::VSHeader;
use color_eyre::eyre::Result;
use object::ReadRef;
use std::mem;

/// Represents a [`Var`](https://docs.microsoft.com/en-us/windows/win32/menurc/var-str) structure.
pub struct VSVar<'data> {
    pub header: VSHeader<'data>,
    pub children: Vec<&'data VarDword>,
}
impl<'data> VSVar<'data> {
    /// Parse a `VSVar` structure at the given [`RVA`](RVA).
    pub fn parse(data: &'data [u8], base_offset: usize) -> Result<Self> {
        let (mut offset, header) = VSHeader::parse(data, base_offset)?;
        let mut consumed = offset;
        offset = align(offset, 4);

        let mut children = Vec::<&'data VarDword>::new();

        while consumed < (*header.length as usize) {
            let child = data.read_at(offset as u64).unwrap();

            offset += mem::size_of::<VarDword>();
            offset = align(offset, 4);
            consumed = offset - base_offset;
            children.push(child);
        }

        Ok(Self { header, children })
    }
}
