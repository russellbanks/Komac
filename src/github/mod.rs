pub mod client;
mod error;
pub mod graphql;
mod rest;
pub mod utils;
mod winget_source;

pub use error::GitHubError;
pub use winget_source::WingetPkgsSource;

pub const MICROSOFT: &str = "microsoft";
pub const WINGET_PKGS: &str = "winget-pkgs";
pub const GITHUB_HOST: &str = "github.com";
