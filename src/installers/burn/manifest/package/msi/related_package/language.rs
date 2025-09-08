use serde::Deserialize;

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L644>
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Language<'manifest> {
    #[serde(rename = "@Id")]
    id: &'manifest str,
}

impl<'manifest> Language<'manifest> {
    /// Returns the language's ID.
    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn id(&self) -> &'manifest str {
        self.id
    }
}
