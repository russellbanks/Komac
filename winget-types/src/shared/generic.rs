use serde::Deserialize;

use crate::shared::manifest_type::ManifestType;

/// A manifest where the only field is the type of the manifest itself. Useful for deserializing
/// once into this type to determine which manifest to properly deserialize into.
#[derive(Debug, Deserialize)]
pub struct GenericManifest {
    #[serde(rename = "ManifestType")]
    pub r#type: ManifestType,
}
