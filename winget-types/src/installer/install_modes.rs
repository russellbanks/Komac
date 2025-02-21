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
    /// A list of supported installer modes internally represented as bit flags
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct InstallModes: u8 {
        const INTERACTIVE = 1;
        const SILENT = 1 << 1;
        const SILENT_WITH_PROGRESS = 1 << 2;
    }
}

const INTERACTIVE: &str = "interactive";
const SILENT: &str = "silent";
const SILENT_WITH_PROGRESS: &str = "silentWithProgress";

impl Display for InstallModes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::INTERACTIVE => f.write_str("Interactive"),
            Self::SILENT => f.write_str("Silent"),
            Self::SILENT_WITH_PROGRESS => f.write_str("Silent with progress"),
            _ => bitflags::parser::to_writer(self, f),
        }
    }
}

impl Serialize for InstallModes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.iter().count()))?;
        for mode in self.iter() {
            match mode {
                Self::INTERACTIVE => seq.serialize_element(INTERACTIVE)?,
                Self::SILENT => seq.serialize_element(SILENT)?,
                Self::SILENT_WITH_PROGRESS => seq.serialize_element(SILENT_WITH_PROGRESS)?,
                _ => {}
            }
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for InstallModes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct InstallModesVisitor;

        impl<'de> Visitor<'de> for InstallModesVisitor {
            type Value = InstallModes;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a sequence of install mode strings")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut modes = InstallModes::empty();

                while let Some(value) = seq.next_element::<&str>()? {
                    match value {
                        INTERACTIVE => modes |= InstallModes::INTERACTIVE,
                        SILENT => modes |= InstallModes::SILENT,
                        SILENT_WITH_PROGRESS => modes |= InstallModes::SILENT_WITH_PROGRESS,
                        _ => {
                            return Err(de::Error::unknown_variant(
                                value,
                                &[INTERACTIVE, SILENT, SILENT_WITH_PROGRESS],
                            ));
                        }
                    }
                }

                Ok(modes)
            }
        }

        deserializer.deserialize_seq(InstallModesVisitor)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use crate::installer::install_modes::InstallModes;

    #[rstest]
    #[case(
        InstallModes::all(),
        indoc! {"
            - interactive
            - silent
            - silentWithProgress
        "}
    )]
    #[case(
        InstallModes::empty(),
        indoc! {"
            []
        "}
    )]
    #[case(
        InstallModes::SILENT_WITH_PROGRESS | InstallModes::SILENT,
        indoc! {"
            - silent
            - silentWithProgress
        "}
    )]
    #[case(
        InstallModes::INTERACTIVE,
        indoc! {"
            - interactive
        "}
    )]
    fn serialize_install_modes(#[case] modes: InstallModes, #[case] expected: &str) {
        assert_eq!(serde_yaml::to_string(&modes).unwrap(), expected);
    }

    #[rstest]
    #[case(
        indoc! {"
            - interactive
            - silent
            - silentWithProgress
        "},
        InstallModes::all(),
    )]
    #[case(
        indoc! {"
            []
        "},
        InstallModes::empty()
    )]
    #[case(
        indoc! {"
            - silentWithProgress
            - silent
        "},
        InstallModes::SILENT | InstallModes::SILENT_WITH_PROGRESS
    )]
    #[case(
        indoc! {"
            - interactive
        "},
        InstallModes::INTERACTIVE,
    )]
    fn deserialize_install_modes(#[case] input: &str, #[case] expected: InstallModes) {
        assert_eq!(
            serde_yaml::from_str::<InstallModes>(input).unwrap(),
            expected
        );
    }

    #[test]
    fn install_modes_serialize_ordered() {
        let input = indoc! {"
            - silentWithProgress
            - silent
            - interactive
        "};

        let deserialized = serde_yaml::from_str::<InstallModes>(input).unwrap();

        assert_eq!(
            serde_yaml::to_string(&deserialized).unwrap(),
            indoc! {"
                - interactive
                - silent
                - silentWithProgress
            "}
        );
    }
}
