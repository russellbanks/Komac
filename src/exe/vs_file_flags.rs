use bitflags::bitflags;

/// A series of bitflags representing the file flags for the [`VS_FIXEDFILEINFO`](https://docs.microsoft.com/en-us/windows/win32/api/verrsrc/ns-verrsrc-vs_fixedfileinfo)
/// structure.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct VSFileFlags(u32);

bitflags! {
    impl VSFileFlags: u32 {
        const DEBUG = 0x0000_0001;
        const PRERELEASE = 0x0000_0002;
        const PATCHED = 0x0000_0004;
        const PRIVATEBUILD = 0x0000_0008;
        const INFOINFERRED = 0x0000_0010;
        const SPECIALBUILD = 0x0000_0020;
    }
}
