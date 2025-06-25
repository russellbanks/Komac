mod arp;
mod cache_type;
mod chain;
mod container;
mod package;
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
use related_bundle::RelatedBundle;
use serde::Deserialize;
use variable::Variable;
pub use variable::VariableType;
use winget_types::Version;
use yes_no::{YesNoButton, bool_from_yes_no};

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BurnManifest<'manifest> {
    #[serde(rename = "@EngineVersion")]
    pub engine_version: Option<Version>,
    #[serde(rename = "@ProtocolVersion")]
    pub protocol_version: Option<u8>,
    #[serde(rename = "@Win64", deserialize_with = "bool_from_yes_no", default)]
    pub win_64: bool,
    #[serde(rename = "RelatedBundle", default, borrow)]
    pub related_bundles: Vec<RelatedBundle<'manifest>>,
    #[serde(rename = "Container", default, borrow)]
    pub containers: Vec<Container<'manifest>>,
    #[serde(rename = "Variable", default, borrow)]
    pub variables: Vec<Variable<'manifest>>,
    #[serde(rename = "Payload", default, borrow)]
    pub payloads: Vec<Payload<'manifest>>,
    #[serde(borrow)]
    pub registration: Registration<'manifest>,
    #[serde(borrow)]
    pub chain: Chain<'manifest>,
}
