use zerocopy::{Immutable, KnownLayout, TryFromBytes};

/// <https://github.com/NSIS-Dev/nsis/blob/v312/Source/Platform.h#L672>
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum RegType {
    #[doc(alias = "REG_NONE")]
    #[default]
    None = 0_u32.to_le(),

    #[doc(alias = "REG_SZ")]
    String = 1_u32.to_le(),

    #[doc(alias = "REG_EXPAND_SZ")]
    ExpandedString = 2_u32.to_le(),

    #[doc(alias = "REG_BINARY")]
    Binary = 3_u32.to_le(),

    #[doc(alias = "REG_DWORD")]
    DWord = 4_u32.to_le(),

    #[doc(alias = "REG_MULTI_SZ")]
    MultiString = 7_u32.to_le(),
}

impl RegType {
    /// Returns `true` if the registry type is [`None`].
    ///
    /// [`None`]: Self::None
    #[inline]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns `true` if the registry type is [`String`].
    ///
    /// [`String`]: Self::String
    #[inline]
    pub const fn is_string(self) -> bool {
        matches!(self, Self::String)
    }

    /// Returns `true` if the registry type is [`ExpandedString`].
    ///
    /// [`ExpandedString`]: Self::ExpandedString
    pub const fn is_expanded_string(self) -> bool {
        matches!(self, Self::ExpandedString)
    }

    /// Returns `true` if the registry type is [`Binary`].
    ///
    /// [`Binary`]: Self::Binary
    #[inline]
    pub const fn is_binary(self) -> bool {
        matches!(self, Self::Binary)
    }

    /// Returns `true` if the registry type is [`DWord`].
    ///
    /// [`DWord`]: Self::DWord
    #[inline]
    pub const fn is_dword(self) -> bool {
        matches!(self, Self::DWord)
    }

    /// Returns `true` if the registry type is [`MultiString`].
    ///
    /// [`MultiString`]: Self::MultiString
    #[inline]
    pub const fn is_multi_string(self) -> bool {
        matches!(self, Self::MultiString)
    }
}
