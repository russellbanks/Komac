use crate::prompts::Prompt;
use crate::prompts::list::ListPrompt;
use nutype::nutype;

#[nutype(
    validate(not_empty, len_char_max = 512),
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
pub struct FileExtension(String);

impl Prompt for FileExtension {
    const MESSAGE: &'static str = "File extensions:";
}

impl ListPrompt for FileExtension {
    const HELP_MESSAGE: &'static str = "List of file extensions the package could support";
    const MAX_ITEMS: u16 = 512;
}
