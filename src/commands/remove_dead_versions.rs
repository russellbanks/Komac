use crate::commands::utils::{SPINNER_SLOW_TICK_RATE, SPINNER_TICK_RATE};
use crate::credential::{get_default_headers, handle_token};
use crate::github::github_client::GitHub;
use crate::github::graphql::get_branches::PullRequestState;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::types::manifest_type::ManifestTypeWithLocale;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::types::urls::url::DecodedUrl;
use anstream::println;
use bon::builder;
use chrono::TimeDelta;
use clap::Parser;
use color_eyre::eyre::Error;
use color_eyre::Result;
use futures_util::{stream, StreamExt, TryStreamExt};
use indicatif::ProgressBar;
use inquire::Confirm;
use itertools::Itertools;
use owo_colors::OwoColorize;
use reqwest::{Client, StatusCode};
use std::collections::BTreeSet;
use std::fmt::Write;
use std::num::NonZeroUsize;
use std::ops::Sub;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// GitHub has an undocumented limit of 150 pull requests per hour
///
/// <https://github.com/cli/cli/issues/4801#issuecomment-1430651377>
const MAX_PULL_REQUESTS_PER_HOUR: u8 = 150;

/// Minimum delay to not go above 150 pull requests per hour
const HOURLY_RATE_LIMIT_DELAY: Duration = Duration::from_secs(
    TimeDelta::hours(1).num_seconds().unsigned_abs() / MAX_PULL_REQUESTS_PER_HOUR as u64,
);

/// GitHub has an undocumented limit of 20 pull requests per minute
///
/// <https://github.com/cli/cli/issues/4801#issuecomment-1430651377>
const MAX_PULL_REQUESTS_PER_MINUTE: u8 = 20;

/// Minimum delay to not go above 20 pull requests per minute
const PER_MINUTE_RATE_LIMIT_DELAY: Duration = Duration::from_secs(
    TimeDelta::minutes(1).num_seconds().unsigned_abs() / MAX_PULL_REQUESTS_PER_MINUTE as u64,
);

const RESOURCE_MISSING_STATUS_CODES: [StatusCode; 2] = [StatusCode::NOT_FOUND, StatusCode::GONE];

/*
This command is hidden because it's mainly for moderation and could be misused.
If you're reading this, feel free to use it, but please be mindful not to spam winget-pkgs
with unnecessary pull requests.
*/
#[derive(Parser)]
#[clap(alias = "rdv", hide = true)]
pub struct RemoveDeadVersions {
    #[arg()]
    package_identifier: PackageIdentifier,

    /// Check versions lesser than a given version
    #[arg(long)]
    before: Option<PackageVersion>,

    /// Check versions greater than a given version
    #[arg(long)]
    after: Option<PackageVersion>,

    /// Use the per-minute rate limit, potentially hitting the hourly rate limit in 7.5 minutes
    #[arg(long, hide = true)]
    fast: bool,

    /// Automatically create pull requests to remove dead versions without prompting
    #[arg(long, hide = true, env = "CI")]
    auto: bool,

    /// Number of installer URLs to check concurrently
    #[arg(short, long, default_value_t = NonZeroUsize::new(num_cpus::get()).unwrap())]
    concurrent_head_requests: NonZeroUsize,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl RemoveDeadVersions {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token).await?;
        let github = GitHub::new(&token)?;
        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;

        let current_user = github.get_username();
        let winget_pkgs = github.get_winget_pkgs().send();

        let versions = github.get_versions(&self.package_identifier).await?;

        let current_user = current_user.await?;
        let fork = github.get_winget_pkgs().owner(&current_user).send().await?;
        let winget_pkgs = winget_pkgs.await?;

        let rate_limit_delay = if self.fast {
            PER_MINUTE_RATE_LIMIT_DELAY
        } else {
            HOURLY_RATE_LIMIT_DELAY
        };

        // Set a default last PR time to before the rate limit delay to do the first PR immediately
        let mut last_pr_time = Instant::now().sub(rate_limit_delay);

        let progress_bar = ProgressBar::new_spinner();
        progress_bar.enable_steady_tick(SPINNER_TICK_RATE);

        for version in versions.iter().filter(|&version| {
            self.before.as_ref().map_or(true, |before| version < before)
                && self.after.as_ref().map_or(true, |after| version > after)
        }) {
            if progress_bar.is_finished() {
                progress_bar.reset();
                progress_bar.enable_steady_tick(SPINNER_TICK_RATE);
            }
            progress_bar.set_message(format!("Checking {} {version}", self.package_identifier));

            let installer_urls = github
                .get_manifest::<InstallerManifest>(
                    &self.package_identifier,
                    version,
                    ManifestTypeWithLocale::Installer,
                )
                .await?
                .installers
                .into_iter()
                .map(|installer| installer.installer_url)
                .collect::<BTreeSet<_>>();

            let url_statuses = stream::iter(installer_urls)
                .map(|url| {
                    let client = &client;
                    async move {
                        let response = client.head(url.as_str()).send().await?;
                        Ok::<(DecodedUrl, StatusCode), reqwest::Error>((url, response.status()))
                    }
                })
                .buffered(self.concurrent_head_requests.get())
                .try_collect::<Vec<_>>()
                .await?;

            let dead_version = url_statuses
                .iter()
                .all(|(_url, status)| RESOURCE_MISSING_STATUS_CODES.contains(status));

            if !dead_version {
                continue;
            }

            progress_bar.finish_and_clear();

            let confirm = confirm_removal()
                .github(&github)
                .identifier(&self.package_identifier)
                .version(version)
                .auto(self.auto)
                .prompt()
                .await?;

            if !confirm {
                continue;
            }

            let deletion_reason = Self::get_deletion_reason(url_statuses)?;

            let time_since_last_pr = Instant::now().duration_since(last_pr_time);
            if time_since_last_pr < rate_limit_delay {
                let wait_time = rate_limit_delay - time_since_last_pr;
                let wait_pb = ProgressBar::new_spinner()
                    .with_message(format!(
                        "Last pull request was created {time_since_last_pr:?} ago. Waiting for {wait_time:?}",
                    ));
                wait_pb.enable_steady_tick(SPINNER_SLOW_TICK_RATE);
                sleep(wait_time).await;
                wait_pb.finish_and_clear();
            }

            github
                .remove_version()
                .identifier(&self.package_identifier)
                .version(version)
                .reason(deletion_reason)
                .fork_owner(&current_user)
                .fork(&fork)
                .winget_pkgs(&winget_pkgs)
                .send()
                .await?;

            last_pr_time = Instant::now();
        }

        Ok(())
    }

    fn get_deletion_reason(url_statuses: Vec<(DecodedUrl, StatusCode)>) -> Result<String> {
        let mut deletion_reason = String::from("All InstallerUrls returned ");
        if let Ok(status) = url_statuses
            .iter()
            .map(|(_url, status)| status)
            .all_equal_value()
        {
            writeln!(&mut deletion_reason, "`{status}`")?;
            for (url, _status) in url_statuses {
                writeln!(&mut deletion_reason, "- {url}")?;
            }
        } else {
            deletion_reason.push_str("missing resources");
            for (url, status) in url_statuses {
                writeln!(&mut deletion_reason, "- {url} - `{status}`")?;
            }
        }
        Ok(deletion_reason)
    }
}

#[builder(finish_fn = prompt)]
async fn confirm_removal(
    github: &GitHub,
    identifier: &PackageIdentifier,
    version: &PackageVersion,
    auto: bool,
) -> Result<bool> {
    if let Some(pull_request) = github
        .get_existing_pull_request(identifier, version)
        .await?
    {
        if pull_request.state == PullRequestState::Open {
            println!(
                "{identifier} {version} returned {} in all its InstallerUrls but there is already {} pull request for this version that was created on {} at {}.",
                StatusCode::NOT_FOUND.red(),
                pull_request.state,
                pull_request.created_at.date_naive(),
                pull_request.created_at.time()
            );
            return if auto {
                Ok(false)
            } else {
                Confirm::new("Remove anyway?").prompt().map_err(Error::from)
            };
        }
    }

    Ok(auto
        || Confirm::new(&format!(
            "{identifier} {version} returned {} in all its InstallerUrls. Remove?",
            StatusCode::NOT_FOUND.red()
        ))
        .prompt()?)
}
