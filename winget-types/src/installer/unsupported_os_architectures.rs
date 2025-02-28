use std::{
    fmt,
    fmt::{Display, Formatter},
};

use bitflags::bitflags;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, de,
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
};

bitflags! {
    /// A list of unsupported arguments internally represented as bit flags
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct UnsupportedOSArchitecture: u8 {
        const X86 = 1;
        const X64 = 1 << 1;
        const ARM = 1 << 2;
        const ARM64 = 1 << 3;
    }
}

const X86: &str = "x86";
const X64: &str = "x64";
const ARM: &str = "arm";
const ARM64: &str = "arm64";

impl Display for UnsupportedOSArchitecture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::X86 => f.write_str(X86),
            Self::X64 => f.write_str(X64),
            Self::ARM => f.write_str(ARM),
            Self::ARM64 => f.write_str(ARM64),
            _ => bitflags::parser::to_writer(self, f),
        }
    }
}

impl Serialize for UnsupportedOSArchitecture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.iter().count()))?;
        for architecture in self.iter() {
            match architecture {
                Self::X86 => seq.serialize_element(X86)?,
                Self::X64 => seq.serialize_element(X64)?,
                Self::ARM => seq.serialize_element(ARM)?,
                Self::ARM64 => seq.serialize_element(ARM64)?,
                _ => {}
            }
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for UnsupportedOSArchitecture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UnsupportedOSArchitectureVisitor;

        impl<'de> Visitor<'de> for UnsupportedOSArchitectureVisitor {
            type Value = UnsupportedOSArchitecture;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a sequence of unsupported OS architectures")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut unsupported_os_architectures = UnsupportedOSArchitecture::empty();

                while let Some(value) = seq.next_element::<&str>()? {
                    match value {
                        X86 => unsupported_os_architectures |= UnsupportedOSArchitecture::X86,
                        X64 => unsupported_os_architectures |= UnsupportedOSArchitecture::X64,
                        ARM => unsupported_os_architectures |= UnsupportedOSArchitecture::ARM,
                        ARM64 => unsupported_os_architectures |= UnsupportedOSArchitecture::ARM64,
                        _ => {
                            return Err(de::Error::unknown_variant(value, &[X86, X64, ARM, ARM64]));
                        }
                    }
                }

                Ok(unsupported_os_architectures)
            }
        }

        deserializer.deserialize_seq(UnsupportedOSArchitectureVisitor)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use crate::installer::UnsupportedOSArchitecture;

    #[rstest]
    #[case(
        UnsupportedOSArchitecture::all(),
        indoc! {"
            - x86
            - x64
            - arm
            - arm64
        "}
    )]
    #[case(
        UnsupportedOSArchitecture::empty(),
        indoc! {"
            []
        "}
    )]
    #[case(
        UnsupportedOSArchitecture::ARM | UnsupportedOSArchitecture::X86,
        indoc! {"
            - x86
            - arm
        "}
    )]
    #[case(
        UnsupportedOSArchitecture::X64,
        indoc! {"
            - x64
        "}
    )]
    fn serialize_unsupported_os_architectures(
        #[case] unsupported_arch: UnsupportedOSArchitecture,
        #[case] expected: &str,
    ) {
        assert_eq!(serde_yaml::to_string(&unsupported_arch).unwrap(), expected);
    }

    #[rstest]
    #[case(
        indoc! {"
            - x86
            - x64
            - arm
            - arm64
        "},
        UnsupportedOSArchitecture::all(),
    )]
    #[case(
        indoc! {"
            []
        "},
        UnsupportedOSArchitecture::empty()
    )]
    #[case(
        indoc! {"
            - arm64
            - x64
        "},
        UnsupportedOSArchitecture::ARM64 | UnsupportedOSArchitecture::X64,
    )]
    #[case(
        indoc! {"
            - arm
        "},
        UnsupportedOSArchitecture::ARM
    )]
    fn deserialize_unsupported_os_architectures(
        #[case] input: &str,
        #[case] expected: UnsupportedOSArchitecture,
    ) {
        assert_eq!(
            serde_yaml::from_str::<UnsupportedOSArchitecture>(input).unwrap(),
            expected
        );
    }

    #[test]
    fn unsupported_arguments_serialize_ordered() {
        let input = indoc! {"
            - arm64
            - arm
            - x64
            - x86
        "};

        let deserialized = serde_yaml::from_str::<UnsupportedOSArchitecture>(input).unwrap();

        assert_eq!(
            serde_yaml::to_string(&deserialized).unwrap(),
            indoc! {"
                - x86
                - x64
                - arm
                - arm64
            "}
        );
    }
}
