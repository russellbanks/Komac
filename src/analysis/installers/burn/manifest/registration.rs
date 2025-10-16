use serde::Deserialize;

use super::{arp::Arp, bool_from_yes_no};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Registration {
    #[serde(rename = "@Code", alias = "@Id")]
    pub id: String,
    #[serde(rename = "@ExecutableName")]
    pub executable_name: String,
    #[serde(rename = "@PerMachine", deserialize_with = "bool_from_yes_no", default)]
    pub per_machine: bool,
    #[serde(rename = "@Tag")]
    pub tag: String,
    #[serde(rename = "@Version")]
    pub version: String,
    #[serde(rename = "@ProviderKey")]
    pub provider_key: String,
    pub arp: Arp,
}

impl Registration {
    #[inline]
    pub const fn id(&self) -> &str {
        self.id.as_str()
    }
}
