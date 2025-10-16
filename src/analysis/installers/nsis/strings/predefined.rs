use std::{borrow::Borrow, fmt};

use zerocopy::{Immutable, KnownLayout, TryFromBytes, ValidityError, try_transmute};

#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromBytes, KnownLayout, Immutable,
)]
#[repr(usize)]
pub enum PredefinedVar {
    CmdLine,
    InstDir,
    OutDir,
    ExeDir,
    Language,
    Temp,
    PluginsDir,
    ExePath,
    ExeFile,
    WindowParent,
    _Click,
    _OutDir,
}

impl PredefinedVar {
    pub const fn num_vars() -> usize {
        Self::all().len()
    }

    pub const fn all() -> &'static [Self; 12] {
        &[
            Self::CmdLine,
            Self::InstDir,
            Self::OutDir,
            Self::ExeDir,
            Self::Language,
            Self::Temp,
            Self::PluginsDir,
            Self::ExePath,
            Self::ExeFile,
            Self::WindowParent,
            Self::_Click,
            Self::_OutDir,
        ]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CmdLine => "$CMDLINE",
            Self::InstDir => "InstallDir",
            Self::OutDir => "$OUTDIR",
            Self::ExeDir => "$EXEDIR",
            Self::Language => "$LANGUAGE",
            Self::Temp => "%Temp%",
            Self::PluginsDir => "Plugins",
            Self::ExePath => "$EXEPATH",
            Self::ExeFile => "$EXEFILE",
            Self::WindowParent => "$HWNDPARENT",
            Self::_Click => "$_CLICK",
            Self::_OutDir => "$_OUTDIR",
        }
    }
}

impl fmt::Display for PredefinedVar {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Borrow<str> for &PredefinedVar {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<usize> for PredefinedVar {
    type Error = ValidityError<usize, Self>;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        try_transmute!(value)
    }
}
