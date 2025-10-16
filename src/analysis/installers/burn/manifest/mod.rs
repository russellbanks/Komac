mod arp;
mod cache_type;
mod chain;
mod container;
pub mod package;
mod payload;
mod registration;
mod related_bundle;
mod variable;
mod yes_no;

use cache_type::BundleCacheType;
use chain::Chain;
use container::Container;
pub use package::{Package, install_condition};
use payload::Payload;
use registration::Registration;
pub use related_bundle::RelatedBundle;
use serde::Deserialize;
use variable::Variable;
pub use variable::VariableType;
use winget_types::Version;
use yes_no::{YesNoButton, bool_from_yes_no};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BurnManifest {
    #[serde(rename = "@EngineVersion")]
    pub engine_version: Option<Version>,
    #[serde(rename = "@ProtocolVersion")]
    pub protocol_version: Option<u8>,
    #[serde(rename = "@Win64", deserialize_with = "bool_from_yes_no", default)]
    pub win_64: bool,
    #[serde(rename = "RelatedBundle", default)]
    pub related_bundles: Vec<RelatedBundle>,
    #[serde(rename = "Container", default)]
    pub containers: Vec<Container>,
    #[serde(rename = "Variable", default)]
    pub variables: Vec<Variable>,
    #[serde(rename = "Payload", default)]
    pub payloads: Vec<Payload>,
    pub registration: Registration,
    pub chain: Chain,
}
