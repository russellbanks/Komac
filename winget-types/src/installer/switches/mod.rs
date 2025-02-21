mod custom;
mod install_location;
mod interactive;
mod log;
mod repair;
mod silent;
mod silent_with_progress;
mod switch;
mod upgrade;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub use crate::installer::switches::{
    custom::CustomSwitch, install_location::InstallLocationSwitch, interactive::InteractiveSwitch,
    log::LogSwitch, repair::RepairSwitch, silent::SilentSwitch,
    silent_with_progress::SilentWithProgressSwitch, upgrade::UpgradeSwitch,
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct InstallerSwitches {
    pub silent: Option<SilentSwitch>,
    pub silent_with_progress: Option<SilentWithProgressSwitch>,
    pub interactive: Option<InteractiveSwitch>,
    pub install_location: Option<InstallLocationSwitch>,
    pub log: Option<LogSwitch>,
    pub upgrade: Option<UpgradeSwitch>,
    pub custom: Option<CustomSwitch>,
    pub repair: Option<RepairSwitch>,
}

impl InstallerSwitches {
    #[must_use]
    pub const fn is_any_some(&self) -> bool {
        self.silent.is_some()
            || self.silent_with_progress.is_some()
            || self.interactive.is_some()
            || self.install_location.is_some()
            || self.log.is_some()
            || self.upgrade.is_some()
            || self.custom.is_some()
    }
}
