use crate::prompts::prompt::OptionalPrompt;
use crate::types::installer_switch::InstallerSwitch;
use nutype::nutype;

#[nutype(derive(
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
))]
pub struct SilentSwitch(InstallerSwitch);

impl OptionalPrompt for SilentSwitch {
    const MESSAGE: &'static str = "Silent installer switch:";
    const HELP_MESSAGE: Option<&'static str> =
        Some("Example: /S, -verysilent, /qn, --silent, /exenoui");
    const PLACEHOLDER: Option<&'static str> = None;
}
