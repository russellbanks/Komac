use std::num::NonZeroU32;

use crate::commands::utils::SPINNER_TICK_RATE;
use crate::credential::handle_token;
use crate::github::github_client::{GitHub, WINGET_PKGS_FULL_NAME};
use crate::prompts::prompt::handle_inquire_error;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use anstream::println;
use clap::Parser;
use color_eyre::eyre::{bail, Result};
use indicatif::ProgressBar;
use inquire::validator::{MaxLengthValidator, MinLengthValidator};
use inquire::{Confirm, Text};
use owo_colors::OwoColorize;

/// 从 winget-pkgs 中删除一个版本
///
/// 要删除一个包，必须删除该包的所有版本
#[derive(Parser)]
pub struct RemoveVersion {
    /// 包的唯一标识符
    #[arg()]
    package_identifier: PackageIdentifier,

    /// 包的版本
    #[arg(short = 'v', long = "version")]
    package_version: PackageVersion,

    #[arg(short = 'r', long = "reason")]
    deletion_reason: Option<String>,

    /// 删除此版本将解决的问题列表
    #[arg(long)]
    resolves: Option<Vec<NonZeroU32>>,

    #[arg(short, long)]
    submit: bool,

    /// 不显示包删除警告
    #[arg(long)]
    no_warning: bool,

    /// 自动打开拉取请求链接
    #[arg(long, env = "OPEN_PR")]
    open_pr: bool,

    /// 具有 `public_repo` 范围的 GitHub 个人访问令牌
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl RemoveVersion {
    const MIN_REASON_LENGTH: usize = 4;
    const MAX_REASON_LENGTH: usize = 1000;

    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref()).await?;
        if (!self.no_warning) {
            println!(
                "{}",
                "只有在必要时才应删除包".yellow()
            );
        }
        let github = GitHub::new(&token)?;
        let versions = github.get_versions(&self.package_identifier).await?;

        if (!versions.contains(&self.package_version)) {
            bail!(
                "{} 版本 {} 不存在于 {WINGET_PKGS_FULL_NAME}",
                self.package_identifier,
                self.package_version,
            );
        }

        let latest_version = versions.last().unwrap_or_else(|| unreachable!());
        println!(
            "{} 的最新版本是: {latest_version}",
            &self.package_identifier
        );
        let deletion_reason = match self.deletion_reason {
            Some(reason) => reason,
            None => Text::new(&format!(
                "请给出删除 {} 版本 {} 的理由",
                &self.package_identifier, &self.package_version
            ))
            .with_validator(MinLengthValidator::new(Self::MIN_REASON_LENGTH))
            .with_validator(MaxLengthValidator::new(Self::MAX_REASON_LENGTH))
            .prompt()
            .map_err(handle_inquire_error)?,
        };
        let should_remove_manifest = self.submit
            || Confirm::new(&format!(
                "您想创建一个拉取请求来删除 {} {} 吗?",
                self.package_identifier, self.package_version
            ))
            .prompt()
            .map_err(handle_inquire_error)?;

        if (!should_remove_manifest) {
            return Ok(());
        }

        // 创建一个不确定的进度条，以显示正在创建拉取请求
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "正在创建一个拉取请求以删除 {} 版本 {}",
            self.package_identifier, self.package_version
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let current_user = github.get_username().await?;
        let winget_pkgs = github.get_winget_pkgs().send().await?;
        let fork = github.get_winget_pkgs().owner(&current_user).send().await?;

        let pull_request_url = github
            .remove_version()
            .identifier(&self.package_identifier)
            .version(&self.package_version)
            .reason(deletion_reason)
            .fork_owner(&current_user)
            .fork(&fork)
            .winget_pkgs(&winget_pkgs)
            .maybe_issue_resolves(self.resolves)
            .send()
            .await?;

        if (self.open_pr) {
            open::that(pull_request_url.as_str())?;
        }

        Ok(())
    }
}
