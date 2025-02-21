use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    X86,
    X64,
    Arm,
    Arm64,
    #[default]
    Neutral,
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum ArchitectureError {
    #[error("Failed to parse as valid Architecture")]
    Invalid,
}

pub const VALID_FILE_EXTENSIONS: [&str; 7] = [
    "msix",
    "msi",
    "appx",
    "exe",
    "zip",
    "msixbundle",
    "appxbundle",
];

const DELIMITERS: [u8; 8] = [b',', b'/', b'\\', b'.', b'_', b'-', b'(', b')'];

const ARCHITECTURES: [(&str, Architecture); 32] = [
    ("x86-64", Architecture::X64),
    ("x86_64", Architecture::X64),
    ("x64", Architecture::X64),
    ("64-bit", Architecture::X64),
    ("64bit", Architecture::X64),
    ("win64a", Architecture::Arm64),
    ("win64", Architecture::X64),
    ("winx64", Architecture::X64),
    ("ia64", Architecture::X64),
    ("amd64", Architecture::X64),
    ("x86", Architecture::X86),
    ("x32", Architecture::X86),
    ("32-bit", Architecture::X86),
    ("32bit", Architecture::X86),
    ("win32", Architecture::X86),
    ("winx86", Architecture::X86),
    ("ia32", Architecture::X86),
    ("i386", Architecture::X86),
    ("i486", Architecture::X86),
    ("i586", Architecture::X86),
    ("i686", Architecture::X86),
    ("386", Architecture::X86),
    ("486", Architecture::X86),
    ("586", Architecture::X86),
    ("686", Architecture::X86),
    ("arm64ec", Architecture::Arm64),
    ("arm64", Architecture::Arm64),
    ("aarch64", Architecture::Arm64),
    ("arm", Architecture::Arm),
    ("armv7", Architecture::Arm),
    ("aarch", Architecture::Arm),
    ("neutral", Architecture::Neutral),
];

impl Architecture {
    #[must_use]
    pub const fn is_64_bit(self) -> bool {
        matches!(self, Self::X64 | Self::Arm64)
    }

    #[must_use]
    pub fn from_url(url: &str) -> Option<Self> {
        // Ignore the casing of the URL
        let url = url.to_ascii_lowercase();

        let url_bytes = url.as_bytes();

        // Check for {delimiter}{architecture}{delimiter}
        for (arch_name, arch) in ARCHITECTURES {
            if let Some(arch_index) = url.rfind(arch_name) {
                // Get characters before and after the architecture
                if let (Some(char_before_arch), Some(char_after_arch)) = (
                    url_bytes.get(arch_index - 1),
                    url_bytes.get(arch_index + arch_name.len()),
                ) {
                    // If the architecture is surrounded by valid delimiters, return the architecture
                    if DELIMITERS.contains(char_before_arch) && DELIMITERS.contains(char_after_arch)
                    {
                        return Some(arch);
                    }
                }
            }
        }

        // If the architecture has not been found, check for {architecture}.{extension}
        for extension in VALID_FILE_EXTENSIONS {
            for (arch_name, arch) in ARCHITECTURES {
                if url
                    .rfind(extension)
                    .map(|index| index - 1)
                    .filter(|&index| url_bytes.get(index) == Some(&b'.'))
                    .is_some_and(|end| url.get(end - arch_name.len()..end) == Some(arch_name))
                {
                    return Some(arch);
                }
            }
        }

        None
    }
}

impl FromStr for Architecture {
    type Err = ArchitectureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86" => Ok(Self::X86),
            "x64" => Ok(Self::X64),
            "arm" => Ok(Self::Arm),
            "arm64" => Ok(Self::Arm64),
            "neutral" => Ok(Self::Neutral),
            _ => Err(Self::Err::Invalid),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::installer::architecture::Architecture;

    #[rstest]
    fn x64_architectures_at_end(
        #[values(
            "x86-64", "x86_64", "x64", "64-bit", "64bit", "Win64", "Winx64", "ia64", "amd64"
        )]
        architecture: &str,
    ) {
        assert_eq!(
            Architecture::from_url(&format!("https://www.example.com/file{architecture}.exe")),
            Some(Architecture::X64)
        );
    }

    #[rstest]
    fn x64_architectures_delimited(
        #[values(
            "x86-64", "x86_64", "x64", "64-bit", "64bit", "Win64", "Winx64", "ia64", "amd64"
        )]
        architecture: &str,
        #[values(',', '/', '\\', '.', '_', '-', '(', ')')] delimiter: char,
    ) {
        assert_eq!(
            Architecture::from_url(&format!(
                "https://www.example.com/file{delimiter}{architecture}{delimiter}app.exe"
            )),
            Some(Architecture::X64)
        );
    }

    #[rstest]
    fn x86_architectures_at_end(
        #[values(
            "x86", "x32", "32-bit", "32bit", "win32", "winx86", "ia32", "i386", "i486", "i586",
            "i686", "386", "486", "586", "686"
        )]
        architecture: &str,
    ) {
        assert_eq!(
            Architecture::from_url(&format!("https://www.example.com/file{architecture}.exe")),
            Some(Architecture::X86)
        );
    }

    #[rstest]
    fn x86_architectures_delimited(
        #[values(
            "x86", "x32", "32-bit", "32bit", "win32", "winx86", "ia32", "i386", "i486", "i586",
            "i686", "386", "486", "586", "686"
        )]
        architecture: &str,
        #[values(',', '/', '\\', '.', '_', '-', '(', ')')] delimiter: char,
    ) {
        assert_eq!(
            Architecture::from_url(&format!(
                "https://www.example.com/file{delimiter}{architecture}{delimiter}app.exe"
            )),
            Some(Architecture::X86)
        );
    }

    #[rstest]
    fn arm64_architectures_at_end(
        #[values("arm64ec", "arm64", "aarch64", "win64a")] architecture: &str,
    ) {
        assert_eq!(
            Architecture::from_url(&format!("https://www.example.com/file{architecture}.exe")),
            Some(Architecture::Arm64)
        );
    }

    #[rstest]
    fn arm64_architectures_delimited(
        #[values("arm64ec", "arm64", "aarch64", "win64a")] architecture: &str,
        #[values(',', '/', '\\', '.', '_', '-', '(', ')')] delimiter: char,
    ) {
        assert_eq!(
            Architecture::from_url(&format!(
                "https://www.example.com/file{delimiter}{architecture}{delimiter}app.exe"
            )),
            Some(Architecture::Arm64)
        );
    }

    #[rstest]
    fn arm_architectures_at_end(#[values("arm", "armv7", "aarch")] architecture: &str) {
        assert_eq!(
            Architecture::from_url(&format!("https://www.example.com/file{architecture}.exe")),
            Some(Architecture::Arm)
        );
    }

    #[rstest]
    fn arm_architectures_delimited(
        #[values("arm", "armv7", "aarch")] architecture: &str,
        #[values(',', '/', '\\', '.', '_', '-', '(', ')')] delimiter: char,
    ) {
        assert_eq!(
            Architecture::from_url(&format!(
                "https://www.example.com/file{delimiter}{architecture}{delimiter}app.exe"
            )),
            Some(Architecture::Arm)
        );
    }

    #[test]
    fn no_architecture() {
        assert_eq!(
            Architecture::from_url("https://www.example.com/file.exe"),
            None
        );
    }
}
