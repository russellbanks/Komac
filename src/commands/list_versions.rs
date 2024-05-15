use crate::credential::handle_token;
use crate::github::github_client::GitHub;
use crate::github::github_client::WINGET_PKGS_FULL_NAME;
use crate::github::utils::get_package_path;
use crate::types::package_identifier::PackageIdentifier;
use clap::{Args, Parser};
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use std::io;
use std::io::Write;

#[derive(Parser)]
pub struct ListVersions {
    #[arg(short = 'i', long = "identifier")]
    package_identifier: PackageIdentifier,

    #[command(flatten)]
    output_type: OutputType,

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
        let token = handle_token(self.token).await?;
        let github = GitHub::new(&token)?;

        let versions = github
            .get_versions(&get_package_path(&self.package_identifier, None))
            .await
            .wrap_err_with(|| {
                format!(
                    "{} does not exist in {WINGET_PKGS_FULL_NAME}",
                    self.package_identifier
                )
            })?;

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
