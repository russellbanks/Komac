use bytemuck::{Pod, Zeroable};

/// Represents a DWORD in the [`VSVar`](VSVar) structure which contains a language ID and a language codepage.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Pod, Zeroable)]
pub struct VarDword {
    lang_id: u16,
    codepage: u16,
}
