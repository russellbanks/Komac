use serde::Deserialize;

#[derive(Copy, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Diverged,
    Ahead,
    Behind,
    Identical,
}

impl Status {
    /// Returns `true` if the commit has diverged from the comparison commit.
    #[expect(unused)]
    #[must_use]
    #[inline]
    pub const fn is_diverged(self) -> bool {
        matches!(self, Self::Diverged)
    }

    /// Returns `true` if the commit is ahead of the comparison commit.
    #[expect(unused)]
    #[must_use]
    #[inline]
    pub const fn is_ahead(self) -> bool {
        matches!(self, Self::Ahead)
    }

    /// Returns `true` if the commit is behind the comparison commit.
    #[expect(unused)]
    #[must_use]
    #[inline]
    pub const fn is_behind(self) -> bool {
        matches!(self, Self::Behind)
    }

    /// Returns `true` if the commit is identical to the comparison commit.
    #[must_use]
    #[inline]
    pub const fn is_identical(self) -> bool {
        matches!(self, Self::Identical)
    }
}
