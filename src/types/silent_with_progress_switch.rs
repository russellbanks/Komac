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
pub struct SilentWithProgressSwitch(String);

impl Prompt for SilentWithProgressSwitch {
    const MESSAGE: &'static str = "带进度的静默安装开关:";
    const HELP_MESSAGE: Option<&'static str> = Some("例如: /S, -silent, /qb, /exebasicui");
    const PLACEHOLDER: Option<&'static str> = None;
}
