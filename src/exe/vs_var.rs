use crate::exe::utils::align;
use crate::exe::var_dword::VarDword;
use crate::exe::vs_header::VSHeader;
use color_eyre::eyre::Result;
use object::ReadRef;

/// Represents a [`Var`](https://docs.microsoft.com/en-us/windows/win32/menurc/var-str) structure.
pub struct VSVar<'data> {
    pub header: VSHeader<'data>,
    pub children: Vec<&'data VarDword>,
}
impl<'data> VSVar<'data> {
    /// Parse a `VSVar` structure at the given [`RVA`](RVA).
    pub fn parse<R: ReadRef<'data>>(data: R, base_offset: u64) -> Result<Self> {
        let (mut offset, header) = VSHeader::parse(data, base_offset)?;
        let mut consumed = offset;
        offset = align(offset, 4);

        let mut children = Vec::<&'data VarDword>::new();

        while consumed < u64::from(*header.length) {
            let child = data.read(&mut offset).unwrap();

            offset = align(offset, 4);
            consumed = offset - base_offset;
            children.push(child);
        }

        Ok(Self { header, children })
    }
}
