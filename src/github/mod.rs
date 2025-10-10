use const_format::formatcp;

pub mod github_client;
pub mod graphql;
mod rest;
pub mod utils;

pub const MICROSOFT: &str = "microsoft";
pub const WINGET_PKGS: &str = "winget-pkgs";
pub const WINGET_PKGS_FULL_NAME: &str = formatcp!("{MICROSOFT}/{WINGET_PKGS}");
pub const GITHUB_HOST: &str = "github.com";
