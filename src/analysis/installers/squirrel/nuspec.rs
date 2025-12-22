use serde::Deserialize;

/// <https://learn.microsoft.com/nuget/reference/nuspec>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct NuSpec {
    pub metadata: Metadata,
}

impl NuSpec {
    /// Returns a human-friendly title of the package which may be used in some UI displays.
    #[must_use]
    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.metadata.title()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub id: String,
    pub version: String,
    pub description: String,
    pub authors: String,
    owners: Option<String>,

    // Some forks don't require title
    pub title: Option<String>,

    // Velopack
    pub main_exe: Option<String>,
    // TODO pass releaseNotes to create/update commands
}

impl Metadata {
    /// Returns a human-friendly title of the package which may be used in some UI displays.
    #[must_use]
    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
}
