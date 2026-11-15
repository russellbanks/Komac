//! Checks GitHub for a newer Komac release and notifies the user on stderr.
//!
//! The check runs in the background while a command executes and is cached
//! for 24 hours. It is skipped entirely in CI, when stderr is not a terminal,
//! or when [`DISABLE_ENV`] is set.

mod cache;
mod notice;

use std::{
    env,
    io::{IsTerminal, stderr},
    time::Duration,
};

use anstream::eprintln;
use chrono::Utc;
use clap::crate_version;
use color_eyre::eyre::Result;
use secrecy::SecretString;
use tokio::task::JoinHandle;
use tracing::debug;

use self::{
    cache::{CachedRelease, ReleaseCache},
    notice::UpdateNotice,
};
use crate::{
    commands::{Commands, utils::environment::CI},
    github::{KOMAC, KOMAC_OWNER, client::GitHub},
};

/// Set to any value to disable update notifications.
const DISABLE_ENV: &str = "KOMAC_NO_UPDATE_NOTIFIER";

/// How long to wait for an unfinished check once the command has completed.
const GRACE_PERIOD: Duration = Duration::from_secs(1);

/// A background check for a newer Komac release.
pub struct UpdateNotifier(Option<JoinHandle<Result<CachedRelease>>>);

impl UpdateNotifier {
    /// Starts checking for a newer release in the background, unless update
    /// notifications are disabled or unsuitable for `command`.
    pub fn start(command: &Commands) -> Self {
        Self(is_enabled(command).then(|| tokio::spawn(latest_release())))
    }

    /// Waits briefly for the check to finish and prints a notice if a newer
    /// release is available. Failures are logged at debug level only, so they
    /// never affect the command's outcome.
    pub async fn finish(self) {
        let Some(handle) = self.0 else {
            return;
        };

        let release = match tokio::time::timeout(GRACE_PERIOD, handle).await {
            Ok(Ok(Ok(release))) => release,
            Ok(Ok(Err(error))) => return debug!("Update check failed: {error}"),
            Ok(Err(error)) => return debug!("Update check task failed: {error}"),
            Err(_) => return debug!("Update check did not finish in time"),
        };

        if let Some(notice) = UpdateNotice::new(crate_version!(), &release) {
            eprintln!();
            eprintln!("{notice}");
        }
    }
}

fn is_enabled(command: &Commands) -> bool {
    env::var_os(DISABLE_ENV).is_none()
        && !*CI
        && stderr().is_terminal()
        && !matches!(command, Commands::Complete(_))
}

/// Returns the latest release from the cache if it is fresh, or from GitHub
/// otherwise.
async fn latest_release() -> Result<CachedRelease> {
    let cache = ReleaseCache::new();

    if let Some(cache) = &cache
        && let Some(release) = cache.load().await
        && release.is_fresh(Utc::now())
    {
        return Ok(release);
    }

    let token = env::var("GITHUB_TOKEN")
        .ok()
        .filter(|token| !token.is_empty())
        .map(SecretString::from);

    let release = GitHub::with_optional_token(token.as_ref())?
        .get_latest_release(KOMAC_OWNER, KOMAC)
        .await?;

    let release = CachedRelease::new(release, Utc::now());

    if let Some(cache) = &cache
        && let Err(error) = cache.store(&release).await
    {
        debug!("Failed to cache latest release: {error}");
    }

    Ok(release)
}
