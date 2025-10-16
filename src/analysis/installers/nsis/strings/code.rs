use std::num::TryFromIntError;

use itertools::Either;
use zerocopy::{Immutable, KnownLayout, TryFromBytes, ValidityError, try_transmute};

use crate::analysis::installers::nsis::version::NsisVersion;

#[allow(dead_code)]
#[derive(Copy, Clone, TryFromBytes, KnownLayout, Immutable)]
#[repr(u8)]
pub enum NsCode {
    LangV3 = 1,
    ShellV3 = 2,
    VarV3 = 3,
    SkipV3 = 4,
    SkipV2 = 252,
    VarV2 = 253,
    ShellV2 = 254,
    LangV2 = 255,
}

impl NsCode {
    pub fn try_new_with_version<T>(code: T, version: NsisVersion) -> Option<Self>
    where
        T: TryInto<Self>,
    {
        code.try_into().ok().filter(|code| code.is_version(version))
    }

    #[inline]
    pub const fn is_lang(self) -> bool {
        matches!(self, Self::LangV2 | Self::LangV3)
    }

    #[inline]
    pub const fn is_shell(self) -> bool {
        matches!(self, Self::ShellV2 | Self::ShellV3)
    }

    #[inline]
    pub const fn is_var(self) -> bool {
        matches!(self, Self::VarV2 | Self::VarV3)
    }

    #[inline]
    pub const fn is_skip(self) -> bool {
        matches!(self, Self::SkipV2 | Self::SkipV3)
    }

    #[inline]
    pub const fn is_v3(self) -> bool {
        matches!(
            self,
            Self::LangV3 | Self::ShellV3 | Self::VarV3 | Self::SkipV3
        )
    }

    #[inline]
    pub const fn is_v2(self) -> bool {
        matches!(
            self,
            Self::LangV2 | Self::ShellV2 | Self::VarV2 | Self::SkipV2
        )
    }

    #[inline]
    pub const fn is_version(self, version: NsisVersion) -> bool {
        (self.is_v3() && version.is_v3()) || (self.is_v2() && version.is_v2())
    }

    pub fn is_code<T>(code: T, version: NsisVersion) -> bool
    where
        T: TryInto<Self>,
    {
        code.try_into().is_ok_and(|code| code.is_version(version))
    }
}

impl TryFrom<u8> for NsCode {
    type Error = ValidityError<u8, Self>;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        try_transmute!(code)
    }
}

impl TryFrom<u16> for NsCode {
    type Error = Either<TryFromIntError, ValidityError<u8, Self>>;

    fn try_from(code: u16) -> Result<Self, Self::Error> {
        u8::try_from(code)
            .map_err(Either::Left)
            .and_then(|code| Self::try_from(code).map_err(Either::Right))
    }
}

#[cfg(test)]
mod tests {
    use super::NsCode;

    #[test]
    fn v3() {
        // V3 codes
        assert!(NsCode::LangV3.is_v3());
        assert!(NsCode::ShellV3.is_v3());
        assert!(NsCode::VarV3.is_v3());
        assert!(NsCode::SkipV3.is_v3());

        // V2 codes
        assert!(!NsCode::SkipV2.is_v3());
        assert!(!NsCode::VarV2.is_v3());
        assert!(!NsCode::ShellV2.is_v3());
        assert!(!NsCode::LangV2.is_v3());
    }

    #[test]
    fn v2() {
        // V2 codes
        assert!(NsCode::LangV2.is_v2());
        assert!(NsCode::ShellV2.is_v2());
        assert!(NsCode::VarV2.is_v2());
        assert!(NsCode::SkipV2.is_v2());

        // V3 codes
        assert!(!NsCode::SkipV3.is_v2());
        assert!(!NsCode::VarV3.is_v2());
        assert!(!NsCode::ShellV3.is_v2());
        assert!(!NsCode::LangV3.is_v2());
    }
}
