pub mod language_tag;
pub mod manifest_type;
pub mod manifest_version;
pub mod package_identifier;
pub mod package_version;
pub mod value;
pub mod version;

const DISALLOWED_CHARACTERS: [char; 9] = ['\\', '/', ':', '*', '?', '\"', '<', '>', '|'];
