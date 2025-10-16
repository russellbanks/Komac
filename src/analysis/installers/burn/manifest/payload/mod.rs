mod packaging;

use packaging::Packaging;
use serde::Deserialize;

use super::bool_from_yes_no;

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L753>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payload {
    #[serde(rename = "@Id")]
    id: String,
    #[serde(rename = "@FilePath")]
    pub file_path: String,
    #[serde(rename = "@FileSize")]
    pub file_size: String,
    #[serde(rename = "@CertificateRootPublicKeyIdentifier")]
    pub certificate_root_public_key_identifier: Option<String>,
    #[serde(rename = "@CertificateRootThumbprint")]
    pub certificate_root_thumbprint: Option<String>,
    #[serde(rename = "@Hash")]
    pub hash: Option<String>,
    #[serde(rename = "@LayoutOnly", deserialize_with = "bool_from_yes_no", default)]
    pub layout_only: bool,
    #[serde(rename = "@DownloadUrl")]
    pub download_url: Option<String>,
    #[serde(rename = "@Packaging", default)]
    pub packaging: Packaging,
    #[serde(rename = "@SourcePath")]
    pub source_path: String,
    #[serde(rename = "@Container")]
    pub container: Option<String>,
}

impl Payload {
    #[inline]
    pub const fn id(&self) -> &str {
        self.id.as_str()
    }
}
