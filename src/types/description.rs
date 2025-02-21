use crate::prompts::Prompt;
use crate::prompts::text::TextPrompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 3, len_char_max = 10000),
    derive(Clone, FromStr, Display, Deserialize, Serialize)
)]
pub struct Description(String);

impl Prompt for Description {
    const MESSAGE: &'static str = "Description:";
}

impl TextPrompt for Description {
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
