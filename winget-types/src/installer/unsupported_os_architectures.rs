use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum UnsupportedOSArchitecture {
    X86,
    X64,
    Arm,
    Arm64,
}
