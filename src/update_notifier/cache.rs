//! On-disk cache of the most recently seen Komac release, so GitHub is queried
//! at most once per [`CHECK_INTERVAL`].

use std::{io, path::PathBuf};

use chrono::{DateTime, TimeDelta, Utc};
use clap::crate_name;
use serde::{Deserialize, Serialize};

use crate::github::Release;

/// How long a cached release stays valid before GitHub is queried again.
const CHECK_INTERVAL: TimeDelta = TimeDelta::hours(24);

const FILE_NAME: &str = "update-check.json";

/// The latest release as of [`checked_at`](Self::checked_at).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedRelease {
    pub checked_at: DateTime<Utc>,
    pub tag_name: String,
    pub url: String,
}

impl CachedRelease {
    pub fn new(release: Release, checked_at: DateTime<Utc>) -> Self {
        Self {
            checked_at,
            tag_name: release.tag_name,
            url: release.html_url,
        }
    }

    /// Returns true if this release was checked recently enough to be reused
    /// without querying GitHub.
    pub fn is_fresh(&self, now: DateTime<Utc>) -> bool {
        now.signed_duration_since(self.checked_at) < CHECK_INTERVAL
    }
}

/// A JSON file in the user's cache directory holding a [`CachedRelease`].
pub struct ReleaseCache {
    path: PathBuf,
}

impl ReleaseCache {
    /// Returns the cache in the platform's cache directory, or `None` if the
    /// platform has no cache directory.
    pub fn new() -> Option<Self> {
        dirs::cache_dir().map(|dir| Self {
            path: dir.join(crate_name!()).join(FILE_NAME),
        })
    }

    /// Loads the cached release. A missing or unreadable cache is treated as
    /// empty.
    pub async fn load(&self) -> Option<CachedRelease> {
        let contents = tokio::fs::read(&self.path).await.ok()?;
        serde_json::from_slice(&contents).ok()
    }

    /// Stores the release, creating the cache directory if needed.
    pub async fn store(&self, release: &CachedRelease) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&self.path, serde_json::to_vec(release)?).await
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn cached_release(checked_at: DateTime<Utc>) -> CachedRelease {
        CachedRelease {
            checked_at,
            tag_name: "v2.18.0".to_owned(),
            url: "https://github.com/russellbanks/Komac/releases/tag/v2.18.0".to_owned(),
        }
    }

    #[rstest]
    #[case::just_checked(TimeDelta::zero(), true)]
    #[case::within_interval(TimeDelta::hours(23), true)]
    #[case::at_interval(TimeDelta::hours(24), false)]
    #[case::long_ago(TimeDelta::days(7), false)]
    fn freshness(#[case] age: TimeDelta, #[case] expected: bool) {
        let now = Utc::now();

        assert_eq!(cached_release(now - age).is_fresh(now), expected);
    }

    #[tokio::test]
    async fn round_trips_through_disk() {
        let directory = tempfile::tempdir().unwrap();
        let cache = ReleaseCache {
            path: directory.path().join(crate_name!()).join(FILE_NAME),
        };
        let release = cached_release(Utc::now());

        assert_eq!(cache.load().await, None);

        cache.store(&release).await.unwrap();

        assert_eq!(cache.load().await, Some(release));
    }
}
