use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(
    Clone, Copy, Debug, Display, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
pub enum Platform {
    #[serde(rename = "Windows.Desktop")]
    #[display("Windows.Desktop")]
    WindowsDesktop,
    #[serde(rename = "Windows.Universal")]
    #[display("Windows.Universal")]
    WindowsUniversal,
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum PlatformError {
    #[error("Platform did not match either `Windows.Desktop` or `Windows.Universal`")]
    NoMatch,
}

impl FromStr for Platform {
    type Err = PlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Windows.Desktop" => Ok(Self::WindowsDesktop),
            "Windows.Universal" => Ok(Self::WindowsUniversal),
            _ => Err(Self::Err::NoMatch),
        }
    }
}
