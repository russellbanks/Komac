use serde::Deserialize;

/// <https://github.com/wixtoolset/wix/blob/main/src/api/wix/WixToolset.Data/Symbols/WixRelatedBundleSymbol.cs#L32>
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Action {
    #[default]
    Detect,
    Upgrade,
    Addon,
    Patch,
}
