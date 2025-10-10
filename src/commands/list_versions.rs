use std::io::Write;

use clap::{Args, Parser};
use color_eyre::Result;
use owo_colors::OwoColorize;
use winget_types::PackageIdentifier;

use crate::{github::client::GitHub, token::TokenManager};

/// Lists all versions for a given package
#[derive(Parser)]
#[clap(visible_alias = "list-versions")]
pub struct ListVersions {
    #[arg()]
    package_identifier: PackageIdentifier,

    #[command(flatten)]
    output_type: OutputType,

    /// Output the number of versions the package has
    #[arg(long)]
    count: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

#[derive(Args)]
#[group(multiple = false)]
struct OutputType {
    /// Output the versions as JSON
    #[arg(long)]
    json: bool,

    /// Output the versions as prettified JSON
    #[arg(long)]
    pretty_json: bool,

    /// Output the versions as YAML
    #[arg(long)]
    yaml: bool,
}

impl ListVersions {
    pub async fn run(self) -> Result<()> {
        let token = TokenManager::handle(self.token).await?;
        let github = GitHub::new(&token)?;

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
                "There are {} versions",
                versions.len().blue().bold()
            )?;
        }

        Ok(())
    }
}
