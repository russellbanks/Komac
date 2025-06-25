use serde::Deserialize;

/// <https://github.com/wixtoolset/wix/blob/main/src/api/wix/WixToolset.Data/PackagingType.cs#L5>
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Packaging {
    #[default]
    Unknown,
    Embedded,
    External,
}
