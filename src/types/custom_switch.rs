use crate::prompts::prompt::Prompt;
use derive_more::IntoIterator;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use smallvec::SmallVec;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CustomSwitchError {
    #[error("Custom switch cannot be empty")]
    CannotBeEmpty,
    #[error(
        "Custom switch cannot be more than {} characters long",
        CustomSwitch::MAX_LENGTH
    )]
    TooLong,
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    IntoIterator,
    DeserializeFromStr,
    SerializeDisplay,
)]
pub struct CustomSwitch(
    // Most custom switches only ever have 1 or (rarely) 2 parts
    #[into_iterator(owned, ref, ref_mut)] SmallVec<[String; 2]>,
);

impl CustomSwitch {
    const MAX_LENGTH: usize = 1 << 11;

    const DELIMITERS: [char; 2] = [',', ' '];

    pub fn all_users() -> Self {
        "/ALLUSERS".parse().unwrap()
    }

    pub fn current_user() -> Self {
        "/CURRENTUSER".parse().unwrap()
    }

    pub fn push(&mut self, other: String) {
        self.0.push(other);
    }

    pub fn contains(&self, other: &str) -> bool {
        self.0.iter().any(|this| this.eq_ignore_ascii_case(other))
    }
}

impl Display for CustomSwitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for part in itertools::intersperse(self.0.iter().map(String::as_str), " ") {
            f.write_str(part)?;
        }
        Ok(())
    }
}

impl FromStr for CustomSwitch {
    type Err = CustomSwitchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(CustomSwitchError::CannotBeEmpty);
        } else if s.chars().count() > Self::MAX_LENGTH {
            return Err(CustomSwitchError::TooLong);
        }

        let switches = s
            .split(Self::DELIMITERS)
            .filter(|switch| !switch.is_empty())
            .map(str::to_owned)
            .collect::<SmallVec<_>>();

        Ok(Self(switches))
    }
}

impl Prompt for CustomSwitch {
    const MESSAGE: &'static str = "Custom installer switch:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: /norestart, -norestart");
    const PLACEHOLDER: Option<&'static str> = None;
}

#[cfg(test)]
mod tests {
    use crate::types::custom_switch::{CustomSwitch, CustomSwitchError};
    use const_format::str_repeat;
    use smallvec::{smallvec, SmallVec};

    #[test]
    fn empty_custom_switch() {
        assert_eq!(
            "".parse::<CustomSwitch>().err(),
            Some(CustomSwitchError::CannotBeEmpty)
        );
    }

    #[test]
    fn unicode_custom_switch_max_length() {
        const CUSTOM_SWITCH: &str = str_repeat!("🦀", CustomSwitch::MAX_LENGTH);

        // Ensure that it's character length that's being checked and not byte or UTF-16 length
        assert!(CUSTOM_SWITCH.len() > CustomSwitch::MAX_LENGTH);
        assert!(CUSTOM_SWITCH.encode_utf16().count() > CustomSwitch::MAX_LENGTH);
        assert_eq!(CUSTOM_SWITCH.chars().count(), CustomSwitch::MAX_LENGTH);
        assert!(CUSTOM_SWITCH.parse::<CustomSwitch>().is_ok());
    }

    #[test]
    fn custom_switch_too_long() {
        const CUSTOM_SWITCH: &str = str_repeat!("🦀", CustomSwitch::MAX_LENGTH + 1);

        assert_eq!(
            CUSTOM_SWITCH.parse::<CustomSwitch>().err(),
            Some(CustomSwitchError::TooLong)
        );
    }

    #[test]
    fn delimited_custom_switches_internal_representation() {
        let switches: SmallVec<[_; 2]> = smallvec!["/ALLUSERS".to_owned(), "/NoRestart".to_owned()];

        assert_eq!(
            switches.join(" ").parse::<CustomSwitch>().unwrap().0,
            switches
        );

        assert_eq!(
            switches.join(", ").parse::<CustomSwitch>().unwrap().0,
            switches
        );
    }

    #[test]
    fn custom_switch_to_string() {
        const CUSTOM_SWITCH: &str = "/ALLUSERS, /NoRestart, , -NoRestart";

        assert_eq!(
            CUSTOM_SWITCH.parse::<CustomSwitch>().unwrap().to_string(),
            "/ALLUSERS /NoRestart -NoRestart"
        );
    }

    #[test]
    fn custom_switch_contains() {
        const ALL_USERS: &str = "/ALLUSERS";

        let all_users_switch = ALL_USERS.parse::<CustomSwitch>().unwrap();

        assert!(all_users_switch.contains(ALL_USERS));
        assert!(all_users_switch.contains(&ALL_USERS.to_ascii_lowercase()))
    }

    #[test]
    fn append_custom_switch() {
        const ALL_USERS: &str = "/ALLUSERS";
        const NO_RESTART: &str = "/NoRestart";

        let mut custom_switch = ALL_USERS.parse::<CustomSwitch>().unwrap();

        custom_switch.push(NO_RESTART.to_owned());

        assert!(custom_switch.contains(NO_RESTART));

        assert_eq!(
            custom_switch.to_string(),
            format!("{ALL_USERS} {NO_RESTART}")
        );
    }
}
