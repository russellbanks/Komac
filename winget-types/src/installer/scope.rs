use serde::{Deserialize, Serialize};

use crate::utils::RelativeDir;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    User,
    Machine,
}

impl Scope {
    #[must_use]
    pub fn from_url(url: &str) -> Option<Self> {
        match url.to_ascii_lowercase() {
            url if url.contains("all-users") || url.contains("machine") => Some(Self::Machine),
            url if url.contains("user") => Some(Self::User),
            _ => None,
        }
    }

    #[must_use]
    pub fn from_install_dir(install_dir: &str) -> Option<Self> {
        const USER_INSTALL_DIRS: [&str; 2] = [RelativeDir::APP_DATA, RelativeDir::LOCAL_APP_DATA];
        const MACHINE_INSTALL_DIRS: [&str; 7] = [
            RelativeDir::PROGRAM_FILES_64,
            RelativeDir::PROGRAM_FILES_32,
            RelativeDir::COMMON_FILES_64,
            RelativeDir::COMMON_FILES_32,
            RelativeDir::PROGRAM_DATA,
            RelativeDir::WINDOWS_DIR,
            RelativeDir::SYSTEM_ROOT,
        ];

        USER_INSTALL_DIRS
            .iter()
            .any(|directory| install_dir.starts_with(directory))
            .then_some(Self::User)
            .or_else(|| {
                MACHINE_INSTALL_DIRS
                    .iter()
                    .any(|directory| install_dir.starts_with(directory))
                    .then_some(Self::Machine)
            })
    }
}
