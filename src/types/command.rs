use crate::prompts::list_prompt::ListPrompt;
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

impl ListPrompt for Command {
    const MESSAGE: &'static str = "命令:";
    const HELP_MESSAGE: &'static str = "用于运行软件包的命令或别名的列表";
    const MAX_ITEMS: u16 = 16;
}
