use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum IconResolution {
    Custom,
    #[serde(rename = "16x16")]
    Size16,
    #[serde(rename = "20x20")]
    Size20,
    #[serde(rename = "24x24")]
    Size24,
    #[serde(rename = "30x30")]
    Size30,
    #[serde(rename = "32x32")]
    Size32,
    #[serde(rename = "36x36")]
    Size36,
    #[serde(rename = "40x40")]
    Size40,
    #[serde(rename = "48x48")]
    Size48,
    #[serde(rename = "60x60")]
    Size60,
    #[serde(rename = "64x64")]
    Size64,
    #[serde(rename = "72x72")]
    Size72,
    #[serde(rename = "80x80")]
    Size80,
    #[serde(rename = "96x96")]
    Size96,
    #[serde(rename = "256x256")]
    Size256,
}
