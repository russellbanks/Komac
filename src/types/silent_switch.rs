use crate::prompts::prompt::Prompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 1, len_char_max = 512),
    derive(
        Clone,
        FromStr,
        Debug,
        Display,
        Deserialize,
        Serialize,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        Hash
    )
)]
pub struct SilentSwitch(String);

impl Prompt for SilentSwitch {
    const MESSAGE: &'static str = "静默安装开关:";
    const HELP_MESSAGE: Option<&'static str> =
        Some("例如: /S, -verysilent, /qn, --silent, /exenoui (winget目前不接受仅交互式安装程序)");
    const PLACEHOLDER: Option<&'static str> = None;
}
