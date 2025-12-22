#![expect(unused)]

use serde::Deserialize;

/// <https://learn.microsoft.com/nuget/reference/nuspec>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct NuSpec {
    pub metadata: Metadata,
}

impl NuSpec {
    /// Returns the case-insensitive package identifier.
    #[must_use]
    #[inline]
    pub const fn id(&self) -> &str {
        self.metadata.id()
    }

    /// Returns the version of the package.
    #[must_use]
    #[inline]
    pub const fn version(&self) -> &str {
        self.metadata.version()
    }

    /// Returns a description of the package for UI display.
    #[must_use]
    #[inline]
    pub const fn description(&self) -> &str {
        self.metadata.description()
    }

    /// Returns a comma-separated list of package authors.
    #[must_use]
    #[inline]
    pub const fn authors(&self) -> &str {
        self.metadata.authors()
    }

    /// Returns a human-friendly title of the package which may be used in some UI displays.
    #[must_use]
    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.metadata.title()
    }

    /// Returns the name of the main executable.
    #[must_use]
    #[inline]
    pub fn main_exe(&self) -> Option<&str> {
        self.metadata.main_exe()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// The case-insensitive package identifier, which must be unique across nuget.org or whatever
    /// gallery the package resides in. IDs may not contain spaces or characters that are not valid
    /// for a URL, and generally follow .NET namespace rules. See
    /// [Choosing a unique package identifier] for guidance.
    ///
    /// When uploading a package to nuget.org, the `id` field is limited to 128 characters.
    ///
    /// [Choosing a unique package identifier]: https://learn.microsoft.com/nuget/create-packages/creating-a-package#choose-a-unique-package-identifier-and-setting-the-version-number
    id: String,
    version: String,
    description: String,
    authors: String,
    owners: Option<String>,

    // Some forks don't require title
    title: Option<String>,

    // Velopack
    main_exe: Option<String>,
    // TODO pass releaseNotes to create/update commands
}

impl Metadata {
    /// Returns the case-insensitive package identifier.
    #[must_use]
    #[inline]
    pub const fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the version of the package.
    #[must_use]
    #[inline]
    pub const fn version(&self) -> &str {
        self.version.as_str()
    }

    /// Returns a description of the package for UI display.
    #[must_use]
    #[inline]
    pub const fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns a comma-separated list of package authors.
    #[must_use]
    #[inline]
    pub const fn authors(&self) -> &str {
        self.authors.as_str()
    }

    /// Returns a human-friendly title of the package which may be used in some UI displays.
    #[must_use]
    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the name of the main executable.
    #[must_use]
    #[inline]
    pub fn main_exe(&self) -> Option<&str> {
        self.main_exe.as_deref()
    }
}
