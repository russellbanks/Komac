use serde::Deserialize;

use super::{arp::Arp, bool_from_yes_no};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Registration<'manifest> {
    #[serde(rename = "@Code", alias = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@ExecutableName")]
    pub executable_name: &'manifest str,
    #[serde(rename = "@PerMachine", deserialize_with = "bool_from_yes_no", default)]
    pub per_machine: bool,
    #[serde(rename = "@Tag")]
    pub tag: &'manifest str,
    #[serde(rename = "@Version")]
    pub version: &'manifest str,
    #[serde(rename = "@ProviderKey")]
    pub provider_key: &'manifest str,
    pub arp: Arp<'manifest>,
}
