pub mod client;
mod error;
pub mod graphql;
mod package;
mod repository;
mod rest;
pub mod utils;

pub use error::GitHubError;
pub use repository::{Repository, set_upstream, upstream};
pub use rest::{CompareStatus, Comparison, MergeType, Release};

pub const MICROSOFT: &str = "microsoft";
pub const WINGET_PKGS: &str = "winget-pkgs";
pub const GITHUB_HOST: &str = "github.com";
pub const KOMAC_OWNER: &str = "russellbanks";
pub const KOMAC: &str = "Komac";
