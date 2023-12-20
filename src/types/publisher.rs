use crate::prompts::prompt::RequiredPrompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 2, len_char_max = 256),
    default = "Publisher",
    derive(Clone, Default, FromStr, Display, Deserialize, Serialize)
)]
pub struct Publisher(String);

impl RequiredPrompt for Publisher {
    const MESSAGE: &'static str = "Publisher:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: Microsoft Corporation");
    const PLACEHOLDER: Option<&'static str> = None;
}
