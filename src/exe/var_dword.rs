use object::Pod;

/// Represents a DWORD in the [`VSVar`](VSVar) structure which contains a language ID and a language codepage.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct VarDword {
    lang_id: u16,
    codepage: u16,
}

unsafe impl Pod for VarDword {}
