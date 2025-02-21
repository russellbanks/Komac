use crate::prompts::Prompt;
use crate::prompts::list::ListPrompt;
use nutype::nutype;

#[nutype(
    validate(not_empty, len_char_max = 40),
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
pub struct Command(String);

impl Prompt for Command {
    const MESSAGE: &'static str = "Commands:";
}

impl ListPrompt for Command {
    const HELP_MESSAGE: &'static str = "List of commands or aliases to run the package";
    const MAX_ITEMS: u16 = 16;
}
