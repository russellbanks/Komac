use std::{
    collections::BTreeSet,
    fmt::Write,
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, Instant},
};

use anstream::println;
use bon::builder;
use chrono::TimeDelta;
use clap::Parser;
use color_eyre::{Result, eyre::Error};
use futures_util::{StreamExt, TryFutureExt, TryStreamExt, stream};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use itertools::Itertools;
use owo_colors::OwoColorize;
use reqwest::{Client, StatusCode};
use tokio::{sync::mpsc, time::sleep, try_join};
use winget_types::{
    ManifestTypeWithLocale, PackageIdentifier, PackageVersion, installer::InstallerManifest,
    url::DecodedUrl,
};

use crate::{
    commands::utils::SPINNER_SLOW_TICK_RATE,
    credential::{get_default_headers, handle_token},
    github::{github_client::GitHub, graphql::get_branches::PullRequestState},
    prompts::text::confirm_prompt,
};

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

    /// Number of versions to check concurrently
    #[arg(short, long, default_value_t = NonZeroUsize::new(num_cpus::get()).unwrap())]
    concurrent: NonZeroUsize,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl RemoveDeadVersions {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;

        let (fork, winget_pkgs, versions) = try_join!(
            github
                .get_username()
                .and_then(|current_user| github.get_winget_pkgs().owner(current_user).send()),
            github.get_winget_pkgs().send(),
            github.get_versions(&self.package_identifier)
        )?;

        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;

        let rate_limit_delay = if self.fast {
            PER_MINUTE_RATE_LIMIT_DELAY
        } else {
            HOURLY_RATE_LIMIT_DELAY
        };

        // Set a default last PR time to before the rate limit delay to do the first PR immediately
        let mut last_pr_time = Instant::now() - rate_limit_delay;

        let versions = versions
            .into_iter()
            .filter(|version| {
                self.before.as_ref().is_none_or(|before| version < before)
                    && self.after.as_ref().is_none_or(|after| version > after)
            })
            .collect::<BTreeSet<_>>();

        let multi_progress = MultiProgress::new();
        let overall_progress = multi_progress.add(
            ProgressBar::new(versions.len() as u64).with_style(
                ProgressStyle::default_bar()
                    .template("{wide_bar:.magenta/black} {human_pos}/{human_len}")?
                    .progress_chars("───"),
            ),
        );

        // Manually tick the overall progress bar once so it draws to the terminal
        overall_progress.tick();

        // Create a vec of progress bars so they can be reused rather than destroyed and recreated
        let progress_bars = (0..self.concurrent.get())
            .map(|_| {
                let pb = multi_progress.add(ProgressBar::new_spinner());
                pb.enable_steady_tick(SPINNER_SLOW_TICK_RATE);
                pb
            })
            .collect::<Vec<_>>();

        // Create a channel to send versions with dead URLs to a listener
        let (sender, mut receiver) = mpsc::channel::<(_, Vec<_>)>(1);

        let package_identifier = Arc::new(self.package_identifier);

        // Create a 'listener' task that waits to receive a version to prompt the user for removal
        let listener = tokio::spawn({
            let package_identifier = Arc::clone(&package_identifier);
            let github = github.clone();
            let multi_progress = multi_progress.clone();
            async move {
                while let Some((version, url_statuses)) = receiver.recv().await {
                    multi_progress.set_draw_target(ProgressDrawTarget::hidden());

                    let confirm = confirm_removal()
                        .github(&github)
                        .identifier(&package_identifier)
                        .version(&version)
                        .auto(self.auto)
                        .prompt()
                        .await?;

                    if !confirm {
                        multi_progress.set_draw_target(ProgressDrawTarget::stderr());
                        continue;
                    }

                    let deletion_reason = Self::get_deletion_reason(&url_statuses)?;

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
                        .identifier(&package_identifier)
                        .version(&version)
                        .reason(deletion_reason)
                        .fork(&fork)
                        .winget_pkgs(&winget_pkgs)
                        .send()
                        .await?;

                    last_pr_time = Instant::now();

                    multi_progress.set_draw_target(ProgressDrawTarget::stderr());
                }

                Ok::<_, color_eyre::Report>(())
            }
        });

        let total = versions.len();
        stream::iter(versions)
            .enumerate()
            .map(|(index, version)| {
                let package_identifier = &package_identifier;
                let github = &github;
                let sender = &sender;
                let client = &client;
                let overall_progress = &overall_progress;
                let progress_bar = &progress_bars[index % self.concurrent.get()];
                async move {
                    progress_bar
                        .set_message(format!("Checking {package_identifier} {}", version.blue()));

                    let installer_urls = github
                        .get_manifest::<InstallerManifest>(
                            package_identifier,
                            &version,
                            ManifestTypeWithLocale::Installer,
                        )
                        .await?
                        .installers
                        .into_iter()
                        .map(|installer| installer.url)
                        .unique();

                    let url_statuses = stream::iter(installer_urls)
                        .map(|url| {
                            client
                                .head((*url).clone())
                                .send()
                                .map_ok(|response| (url, response.status()))
                        })
                        .buffered(2)
                        .try_collect::<Vec<(_, _)>>()
                        .await?;

                    let all_installers_missing = url_statuses
                        .iter()
                        .all(|(_url, status)| RESOURCE_MISSING_STATUS_CODES.contains(status));

                    if all_installers_missing {
                        sender.send((version, url_statuses)).await?;
                    }

                    overall_progress.inc(1);

                    let start = total.saturating_sub(self.concurrent.get()) + 1;
                    if (start..=total).contains(&index) {
                        progress_bar.finish_and_clear();
                    }

                    Ok::<_, color_eyre::Report>(())
                }
            })
            .buffered(self.concurrent.get())
            .try_collect::<()>()
            .await?;

        drop(sender);
        listener.await??;

        multi_progress.clear()?;

        Ok(())
    }

    fn get_deletion_reason(url_statuses: &[(DecodedUrl, StatusCode)]) -> Result<String> {
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
                confirm_prompt("Remove anyway?").map_err(Error::from)
            };
        }
    }

    Ok(auto
        || confirm_prompt(&format!(
            "{identifier} {version} returned {} in all its InstallerUrls. Remove?",
            StatusCode::NOT_FOUND.red()
        ))?)
}
