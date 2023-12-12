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
pub struct SilentWithProgressSwitch(InstallerSwitch);

impl OptionalPrompt for SilentWithProgressSwitch {
    const MESSAGE: &'static str = "Silent with progress installer switch:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: /S, -silent, /qb, /exebasicui");
    const PLACEHOLDER: Option<&'static str> = None;
}
