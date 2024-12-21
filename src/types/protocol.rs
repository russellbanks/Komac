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
    const MESSAGE: &'static str = "协议:";
    const HELP_MESSAGE: &'static str = "软件包提供处理程序的协议列表 (例如: http, https)";
    const MAX_ITEMS: u16 = 16;
}
