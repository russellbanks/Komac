use crate::installers::nsis::version::NsisVersion;
use std::num::NonZeroU8;

#[repr(u8)]
pub enum NsCode {
    Lang,  // 1 if >= NSIS 3 or Unicode, 255 otherwise
    Shell, // 2 or 254
    Var,   // 3 or 253
    Skip,  // 4 or 252
}

impl NsCode {
    pub fn get<T: From<u8>>(self, nsis_version: NsisVersion) -> T {
        let code = if nsis_version.is_v3() {
            NonZeroU8::MIN.get() + self as u8
        } else {
            NonZeroU8::MAX.get() - self as u8
        };
        T::from(code)
    }

    pub fn is_code(code: u8, nsis_version: NsisVersion) -> bool {
        if nsis_version.is_v3() {
            code < Self::Skip.get(nsis_version)
        } else {
            code > Self::Skip.get(nsis_version)
        }
    }
}
