use serde::Deserialize;

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L655>
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PayloadRef<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
}
