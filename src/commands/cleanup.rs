use crate::commands::utils::SPINNER_TICK_RATE;
use crate::credential::handle_token;
use crate::github::github_client::GitHub;
use crate::prompts::prompt::handle_inquire_error;
use anstream::println;
use bitflags::bitflags;
use clap::Parser;
use color_eyre::Result;
use indicatif::ProgressBar;
use inquire::MultiSelect;
use owo_colors::OwoColorize;
use std::fmt::{Display, Formatter};

/// 查找从 winget-pkgs 派生的分支，这些分支有一个已合并或关闭的拉取请求到
/// microsoft/winget-pkgs，提示删除哪些分支
#[derive(Parser)]
#[clap(visible_alias = "clean")]
pub struct Cleanup {
    /// 仅删除已合并的分支
    #[arg(long)]
    only_merged: bool,

    /// 仅删除已关闭的分支
    #[arg(long)]
    only_closed: bool,

    /// 自动删除所有相关分支
    #[arg(short, long, env = "CI")]
    all: bool,

    /// 具有 `public_repo` 范围的 GitHub 个人访问令牌
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl Cleanup {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;

        let merge_state = MergeState::from_bools(self.only_merged, self.only_closed);

        let pb = ProgressBar::new_spinner().with_message(format!(
            "正在检索具有 {merge_state} 拉取请求的分支"
        ));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        // 获取所有具有与 microsoft/winget-pkgs 相关联的拉取请求的派生分支
        let (pr_branch_map, repository_id) = github
            .get_branches(&github.get_username().await?, &merge_state)
            .await?;

        pb.finish_and_clear();

        // 如果没有要删除的分支，则退出
        if pr_branch_map.is_empty() {
            println!(
                "没有 {} 拉取请求的分支可以删除",
                merge_state.blue()
            );
            return Ok(());
        }

        let chosen_pr_branches = if self.all {
            pr_branch_map.keys().collect()
        } else {
            // 显示一个多选提示，选择要删除的分支，所有选项默认选中
            MultiSelect::new(
                "请选择要删除的分支",
                pr_branch_map.keys().collect(),
            )
            .with_all_selected_by_default()
            .with_page_size(10)
            .prompt()
            .map_err(handle_inquire_error)?
        };

        if chosen_pr_branches.is_empty() {
            println!("没有删除任何分支");
            return Ok(());
        }

        // 从选定的拉取请求中获取分支名称
        let branches_to_delete = chosen_pr_branches
            .into_iter()
            .filter_map(|pull_request| pr_branch_map.get(pull_request).map(String::as_str))
            .collect::<Vec<_>>();

        let branch_label = match branches_to_delete.len() {
            1 => "分支",
            _ => "分支",
        };

        pb.reset();
        pb.set_message(format!(
            "正在删除 {} 选定的 {branch_label}",
            branches_to_delete.len(),
        ));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        github
            .delete_branches(&repository_id, &branches_to_delete)
            .await?;

        pb.finish_and_clear();
        println!(
            "{} 已删除 {} 选定的 {branch_label}",
            "成功".green(),
            branches_to_delete.len().blue(),
        );

        Ok(())
    }
}

// Using bitflags instead of an enum to allow combining multiple states (MERGED, CLOSED)
bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct MergeState: u8 {
        const MERGED = 1 << 0;
        const CLOSED = 1 << 1;
    }
}

impl Display for MergeState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::MERGED => "merged",
                Self::CLOSED => "closed",
                _ => "merged or closed",
            }
        )
    }
}

impl MergeState {
    pub const fn from_bools(only_merged: bool, only_closed: bool) -> Self {
        match (only_merged, only_closed) {
            (true, false) => Self::MERGED,
            (false, true) => Self::CLOSED,
            _ => Self::all(),
        }
    }
}
