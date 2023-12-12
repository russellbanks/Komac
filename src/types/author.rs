use crate::prompts::prompt::OptionalPrompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 2, len_char_max = 256),
    derive(Clone, FromStr, Display, Deserialize, Serialize)
)]
pub struct Author(String);

impl OptionalPrompt for Author {
    const MESSAGE: &'static str = "Author:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
