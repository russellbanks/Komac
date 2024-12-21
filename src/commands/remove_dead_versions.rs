use crate::commands::utils::{SPINNER_SLOW_TICK_RATE, SPINNER_TICK_RATE};
use crate::credential::{get_default_headers, handle_token};
use crate::github::github_client::GitHub;
use crate::github::graphql::get_branches::PullRequestState;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::prompts::prompt::handle_inquire_error;
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

/// GitHub 有一个未公开的限制，每小时
/// 
/// <https://github.com/cli/cli/issues/4801#issuecomment-1430651377>
const MAX_PULL_REQUESTS_PER_HOUR: u8 = 150;

/// 最小延迟以不超过每小时 150 个拉取请求
const HOURLY_RATE_LIMIT_DELAY: Duration = Duration::from_secs(
    TimeDelta::hours(1).num_seconds().unsigned_abs() / MAX_PULL_REQUESTS_PER_HOUR as u64,
);

/// GitHub 有一个未公开的限制，每分钟
/// 
/// <https://github.com/cli/cli/issues/4801#issuecomment-1430651377>
const MAX_PULL_REQUESTS_PER_MINUTE: u8 = 20;

/// 最小延迟以不超过每分钟 20 个拉取请求
const PER_MINUTE_RATE_LIMIT_DELAY: Duration = Duration::from_secs(
    TimeDelta::minutes(1).num_seconds().unsigned_abs() / MAX_PULL_REQUESTS_PER_MINUTE as u64,
);

const RESOURCE_MISSING_STATUS_CODES: [StatusCode; 2] = [StatusCode::NOT_FOUND, StatusCode::GONE];

/*
此命令是隐藏的，因为它主要用于管理，可能会被滥用。
如果你正在阅读这段文字，请随意使用，但请注意不要向 winget-pkgs 提交不必要的拉取请求。
*/
#[derive(Parser)]
#[clap(alias = "rdv", hide = true)]
pub struct RemoveDeadVersions {
    #[arg()]
    package_identifier: PackageIdentifier,

    /// 检查小于给定版本的版本
    #[arg(long)]
    before: Option<PackageVersion>,

    /// 检查大于给定版本的版本
    #[arg(long)]
    after: Option<PackageVersion>,

    /// 使用每分钟速率限制，可能在 7.5 分钟内达到每小时速率限制
    #[arg(long, hide = true)]
    fast: bool,

    /// 自动创建拉取请求以删除死版本而无需提示
    #[arg(long, hide = true, env = "CI")]
    auto: bool,

    /// 并发检查安装程序 URL 的数量
    #[arg(short, long, default_value_t = NonZeroUsize::new(num_cpus::get()).unwrap())]
    concurrent_head_requests: NonZeroUsize,

    /// 具有 `public_repo` 范围的 GitHub 个人访问令牌
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl RemoveDeadVersions {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref()).await?;
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

        // 设置默认的最后 PR 时间为速率限制延迟之前，以便立即进行第一次 PR
        let mut last_pr_time = Instant::now().sub(rate_limit_delay);

        let progress_bar = ProgressBar::new_spinner();
        progress_bar.enable_steady_tick(SPINNER_TICK_RATE);

        for version in versions.iter().filter(|&version| {
            self.before.as_ref().is_none_or(|before| version < before)
                && self.after.as_ref().is_none_or(|after| version > after)
        }) {
            if progress_bar.is_finished() {
                progress_bar.reset();
                progress_bar.enable_steady_tick(SPINNER_TICK_RATE);
            }
            progress_bar.set_message(format!("正在检查 {} {version}", self.package_identifier));

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
                        "上次拉取请求创建于 {time_since_last_pr:?} 之前。等待 {wait_time:?}",
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
        let mut deletion_reason = String::from("所有 InstallerUrls 返回 ");
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
            deletion_reason.push_str("缺少资源");
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
                "{identifier} {version} 在所有 InstallerUrls 中返回 {}，但已经有一个 {} 拉取请求，该请求创建于 {} {}。",
                StatusCode::NOT_FOUND.red(),
                pull_request.state,
                pull_request.created_at.date_naive(),
                pull_request.created_at.time()
            );
            return if auto {
                Ok(false)
            } else {
                Confirm::new("仍然删除？")
                    .prompt()
                    .map_err(handle_inquire_error)
                    .map_err(Error::from)
            };
        }
    }

    Ok(auto
        || Confirm::new(&format!(
            "{identifier} {version} 在所有 InstallerUrls 中返回 {}。删除？",
            StatusCode::NOT_FOUND.red()
        ))
        .prompt()
        .map_err(handle_inquire_error)?)
}
