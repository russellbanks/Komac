use crate::commands::utils::SPINNER_TICK_RATE;
use crate::credential::handle_token;
use crate::github::github_client::GitHub;
use crate::hyperlink::Hyperlink;
use anstream::println;
use clap::Parser;
use color_eyre::Result;
use indicatif::ProgressBar;
use owo_colors::OwoColorize;

/// 将更改从 microsoft/winget-pkgs 合并到 fork 仓库
#[derive(Parser)]
#[clap(visible_aliases = ["sync", "merge-upstream"])]
pub struct SyncFork {
    /// 即使 fork 的默认分支不是快进模式，也会合并更改。这不推荐，因为你应该有一个干净的默认分支，
    /// 该分支没有与上游默认分支分叉
    #[arg(short, long)]
    force: bool,

    /// 具有 `public_repo` 范围的 GitHub 个人访问令牌
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl SyncFork {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;

        // 异步获取上游和 fork 仓库的数据
        let winget_pkgs = github.get_winget_pkgs().send();
        let winget_pkgs_fork = github
            .get_winget_pkgs()
            .owner(&github.get_username().await?)
            .send()
            .await?;
        let winget_pkgs = winget_pkgs.await?;

        // 创建指向仓库 URL 的超链接，当打印它们的全名时
        let winget_pkgs_hyperlink = winget_pkgs.full_name.hyperlink(winget_pkgs.url);
        let winget_pkgs_fork_hyperlink = winget_pkgs_fork.full_name.hyperlink(winget_pkgs_fork.url);

        // 检查 fork 是否已经与上游同步，通过它们的最新提交 OID
        if winget_pkgs.default_branch_oid == winget_pkgs_fork.default_branch_oid {
            println!(
                "{} 已经与 {} {}",
                winget_pkgs_fork_hyperlink.blue(),
                winget_pkgs_hyperlink.blue(),
                "同步".green()
            );
            return Ok(());
        }

        // 计算上游领先 fork 的提交数量
        let new_commits_count = winget_pkgs.commit_count - winget_pkgs_fork.commit_count;
        let commit_label = match new_commits_count {
            1 => "提交",
            _ => "提交",
        };

        // 在上游更改合并时显示不确定的进度条
        let pb = ProgressBar::new_spinner().with_message(format!(
            "将 {} 个上游 {} 从 {} 合并到 {}",
            new_commits_count,
            commit_label,
            winget_pkgs.full_name.as_str().blue(),
            winget_pkgs_fork.full_name.as_str().blue(),
        ));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        github
            .merge_upstream(
                &winget_pkgs_fork.default_branch_ref_id,
                winget_pkgs.default_branch_oid,
                self.force,
            )
            .await?;

        pb.finish_and_clear();
        println!(
            "{} 将 {} 个上游 {} 从 {} 合并到 {}",
            "成功".green(),
            new_commits_count,
            commit_label,
            winget_pkgs_hyperlink.blue(),
            winget_pkgs_fork_hyperlink.blue()
        );

        Ok(())
    }
}
