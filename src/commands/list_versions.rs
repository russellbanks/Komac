use std::io::Write;

use clap::{Args, Parser};
use color_eyre::Result;
use secrecy::SecretString;
use winget_types::PackageIdentifier;

use crate::{github::client::GitHub, i18n::fl, token::TokenManager};

#[derive(Parser)]
#[command(about = fl!("command-list"))]
#[clap(visible_alias = "list-versions")]
pub struct ListVersions {
    #[arg()]
    package_identifier: PackageIdentifier,

    #[command(flatten)]
    output_type: OutputType,

    #[arg(long, help = fl!("arg-list-count"))]
    count: bool,

    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true, help = fl!("arg-token"))]
    token: Option<SecretString>,
}

#[derive(Args)]
#[group(multiple = false)]
struct OutputType {
    #[arg(long, help = fl!("arg-list-json"))]
    json: bool,

    #[arg(long, help = fl!("arg-list-pretty-json"))]
    pretty_json: bool,

    #[arg(long, help = fl!("arg-list-yaml"))]
    yaml: bool,
}

impl ListVersions {
    pub async fn run(self) -> Result<()> {
        let token_manager = TokenManager::handle(self.token).await?;
        let github = GitHub::new(token_manager)?;

        let versions = github.get_versions(&self.package_identifier).await?;

        let mut stdout_lock = anstream::stdout().lock();
        match self.output_type {
            OutputType {
                pretty_json: true, ..
            } => serde_json::to_writer_pretty(&mut stdout_lock, &versions)?,
            OutputType { json: true, .. } => serde_json::to_writer(&mut stdout_lock, &versions)?,
            OutputType { yaml: true, .. } => serde_yaml::to_writer(&mut stdout_lock, &versions)?,
            _ => {
                for version in &versions {
                    writeln!(&mut stdout_lock, "{version}")?;
                }
            }
        }

        if self.count {
            writeln!(
                stdout_lock,
                "{}",
                fl!("version-count", count = versions.len())
            )?;
        }

        Ok(())
    }
}
