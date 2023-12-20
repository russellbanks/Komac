use crate::prompts::prompt::RequiredPrompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 3, len_char_max = 512),
    default = "License",
    derive(Clone, Default, FromStr, Display, Deserialize, Serialize)
)]
pub struct License(String);

impl RequiredPrompt for License {
    const MESSAGE: &'static str = "License:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: MIT, GPL-3.0, Freeware, Proprietary");
    const PLACEHOLDER: Option<&'static str> = None;
}
