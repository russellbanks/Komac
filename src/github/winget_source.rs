use std::fmt;

use clap::Args;

use super::{MICROSOFT, WINGET_PKGS};

#[derive(Args, Clone, Debug, PartialEq, Eq)]
#[group(multiple = false)]
pub struct WingetPkgsSource {
    #[arg(long, env, default_value = MICROSOFT)]
    source_owner: String,
    #[arg(long, env, default_value = WINGET_PKGS)]
    source_repo: String,
}

impl WingetPkgsSource {
    /// Creates a new WingetPkgs source from an owner and a repository name.
    pub fn new<O, R>(owner: O, repo: R) -> Self
    where
        O: Into<String>,
        R: Into<String>,
    {
        Self {
            source_owner: owner.into(),
            source_repo: repo.into(),
        }
    }

    #[inline]
    pub const fn owner(&self) -> &str {
        self.source_owner.as_str()
    }

    #[inline]
    pub const fn repo(&self) -> &str {
        self.source_repo.as_str()
    }

    /// Returns `true` if this `winget-pkgs` source is an alternative to the default
    /// `microsoft/winget-pkgs`.
    #[expect(unused)]
    pub fn is_alternative(&self) -> bool {
        self.owner() != MICROSOFT || self.repo() != WINGET_PKGS
    }
}

impl Default for WingetPkgsSource {
    fn default() -> Self {
        Self::new(MICROSOFT, WINGET_PKGS)
    }
}

impl fmt::Display for WingetPkgsSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner(), self.repo())
    }
}
