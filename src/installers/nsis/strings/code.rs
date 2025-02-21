use std::num::NonZeroU8;

use crate::installers::nsis::version::NsisVersion;

#[repr(u8)]
pub enum NsCode {
    Lang,  // 1 if >= NSIS 3, 255 otherwise
    Shell, // 2 or 254
    Var,   // 3 or 253
    Skip,  // 4 or 252
}

impl NsCode {
    pub const fn get(self, nsis_version: NsisVersion) -> u8 {
        if nsis_version.is_v3() {
            NonZeroU8::MIN.get() + self as u8
        } else {
            NonZeroU8::MAX.get() - self as u8
        }
    }

    pub const fn is_code(code: u8, nsis_version: NsisVersion) -> bool {
        if nsis_version.is_v3() {
            code <= Self::Skip.get(nsis_version)
        } else {
            code >= Self::Skip.get(nsis_version)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::installers::nsis::{strings::code::NsCode, version::NsisVersion};

    #[test]
    fn lang() {
        assert_eq!(NsCode::Lang.get(NsisVersion::_2), 255);
        assert_eq!(NsCode::Lang.get(NsisVersion::_3), 1);
    }

    #[test]
    fn shell() {
        assert_eq!(NsCode::Shell.get(NsisVersion::_2), 254);
        assert_eq!(NsCode::Shell.get(NsisVersion::_3), 2);
    }

    #[test]
    fn var() {
        assert_eq!(NsCode::Var.get(NsisVersion::_2), 253);
        assert_eq!(NsCode::Var.get(NsisVersion::_3), 3);
    }

    #[test]
    fn skip() {
        assert_eq!(NsCode::Skip.get(NsisVersion::_2), 252);
        assert_eq!(NsCode::Skip.get(NsisVersion::_3), 4);
    }
}
