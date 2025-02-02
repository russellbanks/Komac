use crate::prompts::prompt::Prompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 1, len_char_max = 2048),
    derive(
        Clone,
        FromStr,
        Debug,
        Display,
        Deserialize,
        Serialize,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        Hash
    )
)]
pub struct CustomSwitch(String);

impl CustomSwitch {
    pub fn all_users() -> Self {
        Self::try_new(String::from("/ALLUSERS")).unwrap()
    }

    pub fn current_user() -> Self {
        Self::try_new(String::from("/CURRENTUSER")).unwrap()
    }
}

impl Prompt for CustomSwitch {
    const MESSAGE: &'static str = "Custom installer switch:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: /norestart, -norestart");
    const PLACEHOLDER: Option<&'static str> = None;
}
