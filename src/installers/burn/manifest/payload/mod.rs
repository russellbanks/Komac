mod packaging;

use packaging::Packaging;
use serde::Deserialize;

use super::bool_from_yes_no;

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L753>
#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payload<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@FilePath")]
    pub file_path: &'manifest str,
    #[serde(rename = "@FileSize")]
    pub file_size: &'manifest str,
    #[serde(rename = "@CertificateRootPublicKeyIdentifier")]
    pub certificate_root_public_key_identifier: Option<&'manifest str>,
    #[serde(rename = "@CertificateRootThumbprint")]
    pub certificate_root_thumbprint: Option<&'manifest str>,
    #[serde(rename = "@Hash")]
    pub hash: Option<&'manifest str>,
    #[serde(rename = "@LayoutOnly", deserialize_with = "bool_from_yes_no", default)]
    pub layout_only: bool,
    #[serde(rename = "@DownloadUrl")]
    pub download_url: Option<&'manifest str>,
    #[serde(rename = "@Packaging", default)]
    pub packaging: Packaging,
    #[serde(rename = "@SourcePath")]
    pub source_path: &'manifest str,
    #[serde(rename = "@Container")]
    pub container: Option<&'manifest str>,
}
