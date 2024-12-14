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
    const MESSAGE: &'static str = "Silent installer switch:";
    const HELP_MESSAGE: Option<&'static str> =
        Some("Example: /S, -verysilent, /qn, --silent, /exenoui");
    const PLACEHOLDER: Option<&'static str> = None;
}
