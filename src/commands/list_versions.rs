use std::io;
use std::io::Write;

use clap::{Args, Parser};
use color_eyre::Result;

use crate::credential::handle_token;
use crate::github::github_client::GitHub;
use crate::types::package_identifier::PackageIdentifier;

/// 列出给定包的所有版本
#[derive(Parser)]
#[clap(visible_alias = "list")]
pub struct ListVersions {
    #[arg()]
    package_identifier: PackageIdentifier,

    #[command(flatten)]
    output_type: OutputType,

    /// 具有 `public_repo` 范围的 GitHub 个人访问令牌
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

#[derive(Args)]
#[group(multiple = false)]
struct OutputType {
    /// 以 JSON 格式输出版本
    #[arg(long)]
    json: bool,

    /// 以美化的 JSON 格式输出版本
    #[arg(long)]
    pretty_json: bool,

    /// 以 YAML 格式输出版本
    #[arg(long)]
    yaml: bool,
}

impl ListVersions {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;

        let versions = github.get_versions(&self.package_identifier).await?;

        let mut stdout_lock = io::stdout().lock();
        match (
            self.output_type.json,
            self.output_type.pretty_json,
            self.output_type.yaml,
        ) {
            (true, _, _) => serde_json::to_writer(stdout_lock, &versions)?,
            (_, true, _) => serde_json::to_writer_pretty(stdout_lock, &versions)?,
            (_, _, true) => serde_yaml::to_writer(stdout_lock, &versions)?,
            _ => {
                for version in versions {
                    writeln!(stdout_lock, "{version}")?;
                }
            }
        }

        Ok(())
    }
}
