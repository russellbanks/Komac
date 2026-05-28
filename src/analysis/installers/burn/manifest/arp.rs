#![expect(unused)]

use serde::Deserialize;
use winget_types::Version;

use super::{YesNoButton, bool_from_yes_no};

/// Attributes for the Programs and Features (also known as Add/Remove Programs).
///
/// <https://github.com/wixtoolset/wix/blob/v7.0.0/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L232>
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Arp {
    #[serde(rename = "@Register", deserialize_with = "bool_from_yes_no", default)]
    pub register: bool,

    #[serde(rename = "@DisplayName")]
    display_name: String,

    #[serde(rename = "@DisplayVersion")]
    display_version: Version,

    /// Optional name to display in Add/Remove Programs while the bundle is being installed,
    /// uninstalled, repaired.
    #[serde(rename = "@InProgressDisplayName")]
    in_progress_display_name: Option<String>,

    #[serde(rename = "@Publisher")]
    publisher: Option<String>,

    /// A URL to the help for the bundle to display in Programs and Features.
    #[serde(rename = "@HelpLink")]
    help_url: Option<String>,

    /// A telephone number for help to display in Programs and Features.
    #[serde(rename = "@HelpTelephone")]
    help_telephone: Option<String>,

    /// A URL for more information about the bundle to display in Programs and Features.
    #[serde(rename = "@AboutUrl")]
    about_url: Option<String>,

    /// A URL for updates of the bundle to display in Programs and Features.
    #[serde(rename = "@UpdateUrl")]
    update_url: Option<String>,

    /// The name of the parent bundle to display in Installed Updates.
    #[serde(rename = "@ParentDisplayName")]
    parent_display_name: Option<String>,

    /// Determines whether the bundle can be modified via the Programs and Features.
    ///
    /// If the value is “button” then Programs and Features will show a single “Uninstall/Change”
    /// button. If the value is “yes” then Programs and Features will only show the “Uninstall”
    /// button”. If the value is “no”, the default, then a “Change” button is shown.
    #[serde(rename = "@DisableModify", default)]
    disable_modify: YesNoButton,

    /// Determines whether the bundle can be removed via the Programs and Features.
    ///
    /// If the value is “yes” then the “Uninstall” button will not be displayed.
    /// The default is “no” which ensures there is an “Uninstall” button to remove the bundle.
    /// If the “`DisableModify`” attribute is also “yes” or “button” then the bundle will not be
    /// displayed in Programs and Features and another mechanism (such as registering as a related
    /// bundle addon) must be used to ensure the bundle can be removed.
    #[serde(
        rename = "@DisableRemove",
        deserialize_with = "bool_from_yes_no",
        default
    )]
    disable_remove: bool,
}

impl Arp {
    /// Returns the display name.
    #[inline]
    pub const fn display_name(&self) -> &str {
        self.display_name.as_str()
    }

    /// Returns the publisher of the bundle to display in Programs and Features , or `None` if not
    /// specified.
    #[inline]
    pub fn publisher(&self) -> Option<&str> {
        self.publisher.as_deref()
    }

    #[inline]
    pub const fn display_version(&self) -> &Version {
        &self.display_version
    }

    /// Returns the help URL to display in Programs and Features.
    pub fn help_url(&self) -> Option<&str> {
        self.help_url.as_deref()
    }

    /// Returns the help telephone number to display in Programs and Features, or `None` if not
    /// specified.
    pub fn help_telephone(&self) -> Option<&str> {
        self.help_telephone.as_deref()
    }

    /// Returns the URL for more information about the bundle to display in Programs and Features,
    /// or `None` if not specified.
    pub fn about_url(&self) -> Option<&str> {
        self.about_url.as_deref()
    }

    /// Returns the URL for updates of the bundle to display in Programs and Features, or `None` if
    /// not specified.
    pub fn update_url(&self) -> Option<&str> {
        self.update_url.as_deref()
    }

    /// Returns the parent display name.
    pub fn parent_display_name(&self) -> Option<&str> {
        self.parent_display_name.as_deref()
    }

    #[inline]
    pub const fn disable_modify(&self) -> YesNoButton {
        self.disable_modify
    }

    /// Returns true if the ability to remove the bundle via Programs and Features has been
    /// disabled.
    #[inline]
    pub const fn disable_remove(&self) -> bool {
        self.disable_remove
    }
}
