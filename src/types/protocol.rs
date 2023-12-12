use crate::prompts::list_prompt::ListPrompt;
use nutype::nutype;

#[nutype(
    validate(not_empty, len_char_max = 2048),
    derive(
        Clone,
        Debug,
        Deserialize,
        Display,
        Eq,
        FromStr,
        Ord,
        PartialEq,
        PartialOrd,
        Serialize,
        Hash
    )
)]
pub struct Protocol(String);

impl ListPrompt for Protocol {
    const MESSAGE: &'static str = "Protocols:";
    const HELP_MESSAGE: &'static str = "List of protocols the package provides a handler for";
    const MAX_ITEMS: u16 = 16;
}
