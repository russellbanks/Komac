use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents the file mode used in Git trees.
///
/// Possible values are:
/// - `100644`: regular file (blob)
/// - `100755`: executable file (blob)
/// - `040000`: subdirectory (tree)
/// - `160000`: submodule (commit)
/// - `120000`: symlink (blob)
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitFileMode {
    #[serde(rename = "100644")]
    File = 0o100_644,
    #[serde(rename = "100755")]
    Executable = 0o100_755,
    #[serde(rename = "040000")]
    Directory = 0o040_000,
    #[serde(rename = "160000")]
    Submodule = 0o160_000,
    #[serde(rename = "120000")]
    Symlink = 0o120_000,
}

impl GitFileMode {
    /// Returns `true` if the mode is `100644`.
    #[inline]
    pub const fn is_file(self) -> bool {
        matches!(self, Self::File)
    }

    /// Returns `true` if the mode is `100755`.
    #[inline]
    pub const fn is_executable(self) -> bool {
        matches!(self, Self::Executable)
    }

    /// Returns `true` if the mode is `040000`.
    #[inline]
    pub const fn is_directory(self) -> bool {
        matches!(self, Self::Directory)
    }

    /// Returns `true` if the mode is `160000`.
    #[inline]
    pub const fn is_submodule(self) -> bool {
        matches!(self, Self::Submodule)
    }

    /// Returns `true` if the mode is `120000`.
    #[inline]
    pub const fn is_symlink(self) -> bool {
        matches!(self, Self::Symlink)
    }

    /// Returns the Git file mode as a static string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "100644",
            Self::Executable => "100755",
            Self::Directory => "040000",
            Self::Submodule => "160000",
            Self::Symlink => "120000",
        }
    }
}

impl fmt::Debug for GitFileMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File => write!(f, "File({self})"),
            Self::Executable => write!(f, "Executable({self:o})"),
            Self::Directory => write!(f, "Directory({self:o})"),
            Self::Symlink => write!(f, "Symlink({self:o})"),
            Self::Submodule => write!(f, "Submodule({self:o})"),
        }
    }
}

impl fmt::Display for GitFileMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Octal for GitFileMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Octal::fmt(&(*self as u16), f)
    }
}

impl From<GitFileMode> for u16 {
    /// Converts a Git file mode into a `u16`.
    #[inline]
    fn from(mode: GitFileMode) -> Self {
        mode as Self
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::GitFileMode;

    #[rstest]
    #[case::file(GitFileMode::File, 0o100_644)]
    #[case::executable(GitFileMode::Executable, 0o100_755)]
    #[case::directory(GitFileMode::Directory, 0o040_000)]
    #[case::submodule(GitFileMode::Submodule, 0o160_000)]
    #[case::symlink(GitFileMode::Symlink, 0o1200_00)]
    fn git_file_mode_to_u16(#[case] git_file_mode: GitFileMode, #[case] expected: u16) {
        assert_eq!(u16::from(git_file_mode), expected);
        assert_eq!(u16::from(git_file_mode), git_file_mode as u16);
    }

    #[rstest]
    #[case::file(GitFileMode::File, "0o100644")]
    #[case::executable(GitFileMode::Executable, "0o100755")]
    #[case::directory(GitFileMode::Directory, "0o040000")]
    #[case::submodule(GitFileMode::Submodule, "0o160000")]
    #[case::symlink(GitFileMode::Symlink, "0o120000")]
    fn fmt_octal(#[case] git_file_mode: GitFileMode, #[case] expected: &str) {
        assert_eq!(format!("{git_file_mode:#08o}"), expected);
        assert_eq!(
            format!("{git_file_mode:06o}"),
            expected.trim_start_matches("0o")
        );
    }

    #[rstest]
    #[case(GitFileMode::File, "100644")]
    #[case(GitFileMode::Executable, "100755")]
    #[case(GitFileMode::Directory, "040000")]
    #[case(GitFileMode::Submodule, "160000")]
    #[case(GitFileMode::Symlink, "120000")]
    fn as_str(#[case] mode: GitFileMode, #[case] expected: &str) {
        assert_eq!(mode.as_str(), expected);
    }

    #[test]
    fn predicates() {
        assert!(GitFileMode::File.is_file());
        assert!(!GitFileMode::File.is_executable());
        assert!(GitFileMode::Executable.is_executable());
        assert!(GitFileMode::Directory.is_directory());
        assert!(GitFileMode::Submodule.is_submodule());
        assert!(GitFileMode::Symlink.is_symlink());
    }

    #[rstest]
    #[case(GitFileMode::File, r#""100644""#)]
    #[case(GitFileMode::Executable, r#""100755""#)]
    #[case(GitFileMode::Directory, r#""040000""#)]
    #[case(GitFileMode::Submodule, r#""160000""#)]
    #[case(GitFileMode::Symlink, r#""120000""#)]
    fn serde_roundtrip(#[case] mode: GitFileMode, #[case] expected_json: &str) {
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, expected_json);

        let decoded: GitFileMode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, mode);
    }
}
