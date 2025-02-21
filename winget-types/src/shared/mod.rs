mod generic;
mod language_tag;
mod manifest_type;
mod manifest_version;
mod package_identifier;
mod package_version;
mod sha_256;
pub mod url;
pub mod value;
mod version;

pub use generic::GenericManifest;
pub use language_tag::LanguageTag;
pub use manifest_type::{ManifestType, ManifestTypeWithLocale};
pub use manifest_version::ManifestVersion;
pub use package_identifier::PackageIdentifier;
pub use package_version::PackageVersion;
pub use sha_256::Sha256String;
pub use version::Version;

const DISALLOWED_CHARACTERS: [char; 9] = ['\\', '/', ':', '*', '?', '\"', '<', '>', '|'];
