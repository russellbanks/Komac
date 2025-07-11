use serde::Deserialize;

/// <https://github.com/wixtoolset/wix/blob/main/src/api/wix/WixToolset.Data/Symbols/WixRelatedBundleSymbol.cs#L32>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Action {
    Detect,
    Upgrade,
    Addon,
    Patch,
}
