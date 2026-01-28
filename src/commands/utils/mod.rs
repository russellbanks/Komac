pub mod environment;
mod rate_limit;
mod submit_option;

use std::time::Duration;

use camino::Utf8Path;
use chrono::Local;
use color_eyre::Result;
use futures_util::{StreamExt, TryStreamExt, stream};
use inquire::error::InquireResult;
pub use rate_limit::RateLimit;
pub use submit_option::SubmitOption;
use tokio::{fs, fs::File, io::AsyncWriteExt};
use winget_types::{PackageIdentifier, PackageVersion};

use crate::{
    commands::utils::environment::CI,
    github::{
        graphql::get_existing_pull_request::PullRequest,
        utils::pull_request::print_pull_request_url,
    },
    prompts::text::confirm_prompt,
    terminal::Hyperlinkable,
};

pub const SPINNER_TICK_RATE: Duration = Duration::from_millis(50);

pub const SPINNER_SLOW_TICK_RATE: Duration = Duration::from_millis(100);

pub fn prompt_existing_pull_request(
    identifier: &PackageIdentifier,
    version: &PackageVersion,
    pull_request: &PullRequest,
) -> InquireResult<bool> {
    let created_at = pull_request.created_at.with_timezone(&Local);
    print_pull_request_url(
        &pull_request.url,
        format!(
            "There is already {} {} for {identifier} {version} that was created on {} at {}",
            pull_request.state,
            "pull request".hyperlink(&pull_request.url),
            created_at.date_naive(),
            created_at.time()
        ),
    );
    if *CI {
        // Exit instead of proceeding in CI environments
        Ok(false)
    } else {
        confirm_prompt("Would you like to proceed?")
    }
}

pub async fn write_changes_to_dir(changes: &[(String, String)], output: &Utf8Path) -> Result<()> {
    fs::create_dir_all(output).await?;
    stream::iter(changes.iter())
        .map(|(path, content)| async move {
            if let Some(file_name) = Utf8Path::new(path).file_name() {
                let mut file = File::create(output.join(file_name)).await?;
                file.write_all(content.as_bytes()).await?;
            }
            Ok::<(), color_eyre::eyre::Error>(())
        })
        .buffer_unordered(2)
        .try_collect()
        .await
}
