use crate::prompts::Prompt;
use crate::prompts::list::ListPrompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 1, len_char_max = 40),
    derive(
        Clone,
        FromStr,
        Display,
        Deserialize,
        Serialize,
        Eq,
        PartialEq,
        Ord,
        PartialOrd
    )
)]
pub struct Tag(String);

impl Prompt for Tag {
    const MESSAGE: &'static str = "Tags:";
}

impl ListPrompt for Tag {
    const HELP_MESSAGE: &'static str = "Example: zip, c++, photos, OBS";
    const MAX_ITEMS: u16 = 16;
}
