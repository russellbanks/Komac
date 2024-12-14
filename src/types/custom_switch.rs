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

impl Prompt for CustomSwitch {
    const MESSAGE: &'static str = "Custom installer switch:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: /norestart, -norestart");
    const PLACEHOLDER: Option<&'static str> = None;
}
