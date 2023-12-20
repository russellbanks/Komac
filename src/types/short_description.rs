use crate::prompts::prompt::RequiredPrompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 2, len_char_max = 256),
    default = "Short Description",
    derive(Clone, FromStr, Default, Display, Deserialize, Serialize)
)]
pub struct ShortDescription(String);

impl RequiredPrompt for ShortDescription {
    const MESSAGE: &'static str = "Short description:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
