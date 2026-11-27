//! The message shown when a newer Komac release is available.

use std::fmt;

use clap::crate_name;
use owo_colors::OwoColorize;
use winget_types::Version;

use super::cache::CachedRelease;
use crate::i18n::fl;

/// A notice that a newer release than the running version is available.
#[derive(Debug, PartialEq, Eq)]
pub struct UpdateNotice<'release> {
    current: Version,
    latest: Version,
    url: &'release str,
}

impl<'release> UpdateNotice<'release> {
    /// Returns a notice if `release` is newer than `current`.
    pub fn new(current: &str, release: &'release CachedRelease) -> Option<Self> {
        let current = parse_version(current)?;
        let latest = parse_version(&release.tag_name)?;

        (latest > current).then_some(Self {
            current,
            latest,
            url: &release.url,
        })
    }
}

impl fmt::Display for UpdateNotice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {} → {}",
            fl!("update-available", name = crate_name!()).yellow(),
            self.current.cyan(),
            self.latest.cyan()
        )?;
        write!(f, "{}", self.url.yellow())
    }
}

/// Parses a version from a release tag, which may be prefixed with `v`.
fn parse_version(tag: &str) -> Option<Version> {
    tag.strip_prefix(['v', 'V']).unwrap_or(tag).parse().ok()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::newer_minor("v2.18.0", "2.17.2", true)]
    #[case::newer_major("v3.0.0", "2.17.2", true)]
    #[case::newer_numerically("v2.17.10", "2.17.2", true)]
    #[case::without_prefix("2.18.0", "2.17.2", true)]
    #[case::same("v2.17.2", "2.17.2", false)]
    #[case::older("v2.17.1", "2.17.2", false)]
    fn notice_only_for_newer_releases(
        #[case] tag_name: &str,
        #[case] current: &str,
        #[case] expected: bool,
    ) {
        let release = CachedRelease {
            checked_at: Utc::now(),
            tag_name: tag_name.to_owned(),
            url: String::new(),
        };

        assert_eq!(UpdateNotice::new(current, &release).is_some(), expected);
    }
}
