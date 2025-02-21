use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use compact_str::CompactString;
use derive_more::IntoIterator;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use smallvec::SmallVec;
use thiserror::Error;

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    IntoIterator,
    SerializeDisplay,
    DeserializeFromStr,
)]
pub struct InstallerSwitch<const N: usize>(
    #[into_iterator(owned, ref, ref_mut)] SmallVec<[CompactString; 2]>,
);

impl<const N: usize> InstallerSwitch<N> {
    pub const MAX_LENGTH: usize = N;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SwitchError<const N: usize> {
    #[error("Switch cannot be empty")]
    Empty,
    #[error("Switch cannot be more than {N} characters long")]
    TooLong,
}

impl<const N: usize> InstallerSwitch<N> {
    const DELIMITERS: [char; 2] = [',', ' '];

    pub fn push<S: Into<CompactString>>(&mut self, other: S) {
        self.0.push(other.into());
    }

    pub fn contains<S: AsRef<str>>(&self, other: S) -> bool {
        self.0
            .iter()
            .any(|this| this.eq_ignore_ascii_case(other.as_ref()))
    }
}

impl<const N: usize> Display for InstallerSwitch<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for part in itertools::intersperse(self.0.iter().map(CompactString::as_str), " ") {
            f.write_str(part)?;
        }
        Ok(())
    }
}

impl<const N: usize> FromStr for InstallerSwitch<N> {
    type Err = SwitchError<N>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(SwitchError::Empty)
        } else if s.chars().count() > N {
            Err(SwitchError::TooLong)
        } else {
            Ok(Self(
                s.split(Self::DELIMITERS)
                    .filter(|switch| !switch.is_empty())
                    .map(CompactString::from)
                    .collect::<SmallVec<_>>(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use smallvec::{SmallVec, smallvec};

    use crate::installer::switches::{log::LogSwitch, switch::SwitchError};

    #[test]
    fn empty_custom_switch() {
        assert_eq!("".parse::<LogSwitch>().err().unwrap(), SwitchError::Empty);
    }

    #[test]
    fn unicode_custom_switch_max_length() {
        let custom_switch = "🦀".repeat(LogSwitch::MAX_LENGTH);

        // Ensure that it's character length that's being checked and not byte or UTF-16 length
        assert!(custom_switch.len() > LogSwitch::MAX_LENGTH);
        assert!(custom_switch.encode_utf16().count() > LogSwitch::MAX_LENGTH);
        assert_eq!(custom_switch.chars().count(), LogSwitch::MAX_LENGTH);
        assert!(custom_switch.parse::<LogSwitch>().is_ok());
    }

    #[test]
    fn custom_switch_too_long() {
        let custom_switch = "a".repeat(LogSwitch::MAX_LENGTH + 1);

        assert_eq!(
            custom_switch.parse::<LogSwitch>().err(),
            Some(SwitchError::TooLong)
        );
    }

    #[test]
    fn delimited_custom_switches_internal_representation() {
        let switches: SmallVec<[_; 2]> = smallvec!["/ALLUSERS".to_owned(), "/NoRestart".to_owned()];

        assert_eq!(switches.join(" ").parse::<LogSwitch>().unwrap().0, switches);

        assert_eq!(
            switches.join(", ").parse::<LogSwitch>().unwrap().0,
            switches
        );
    }

    #[test]
    fn custom_switch_to_string() {
        const CUSTOM_SWITCH: &str = "/ALLUSERS, /NoRestart, , -NoRestart";

        assert_eq!(
            CUSTOM_SWITCH.parse::<LogSwitch>().unwrap().to_string(),
            "/ALLUSERS /NoRestart -NoRestart"
        );
    }

    #[test]
    fn custom_switch_contains() {
        const ALL_USERS: &str = "/ALLUSERS";

        let all_users_switch = ALL_USERS.parse::<LogSwitch>().unwrap();

        assert!(all_users_switch.contains(ALL_USERS));
        assert!(all_users_switch.contains(&ALL_USERS.to_ascii_lowercase()))
    }

    #[test]
    fn append_custom_switch() {
        const ALL_USERS: &str = "/ALLUSERS";
        const NO_RESTART: &str = "/NoRestart";

        let mut custom_switch = ALL_USERS.parse::<LogSwitch>().unwrap();

        custom_switch.push(NO_RESTART);

        assert!(custom_switch.contains(NO_RESTART));

        assert_eq!(
            custom_switch.to_string(),
            format!("{ALL_USERS} {NO_RESTART}")
        );
    }
}
