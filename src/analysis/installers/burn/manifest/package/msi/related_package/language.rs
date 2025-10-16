use serde::Deserialize;

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L644>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Language {
    #[serde(rename = "@Id")]
    id: String,
}

impl Language {
    /// Returns the language's ID.
    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn id(&self) -> &str {
        self.id.as_str()
    }
}
