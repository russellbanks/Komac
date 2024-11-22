use anstream::println;
use camino::Utf8Path;
use chrono::Local;
use color_eyre::Result;
use futures_util::{stream, StreamExt, TryStreamExt};
use inquire::{Confirm, Select};
use owo_colors::OwoColorize;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use strum::{Display, EnumIter, IntoEnumIterator};
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::editor::Editor;
use crate::github::graphql::get_existing_pull_request::PullRequest;
use crate::manifests::installer_manifest::AppsAndFeaturesEntry;
use crate::manifests::print_changes;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;

pub const SPINNER_TICK_RATE: Duration = Duration::from_millis(50);

pub const SPINNER_SLOW_TICK_RATE: Duration = Duration::from_millis(100);

pub fn prompt_existing_pull_request(
    identifier: &PackageIdentifier,
    version: &PackageVersion,
    pull_request: &PullRequest,
) -> Result<bool> {
    let created_at = pull_request.created_at.with_timezone(&Local);
    println!(
        "There is already {} pull request for {identifier} {version} that was created on {} at {}",
        pull_request.state,
        created_at.date_naive(),
        created_at.time()
    );
    println!("{}", pull_request.url.blue());
    let proceed = if env::var("CI").is_ok_and(|ci| bool::from_str(&ci) == Ok(true)) {
        false
    } else {
        Confirm::new("Would you like to proceed?").prompt()?
    };
    Ok(proceed)
}

pub fn prompt_submit_option(
    changes: &mut [(String, String)],
    submit: bool,
    identifier: &PackageIdentifier,
    version: &PackageVersion,
    dry_run: bool,
) -> Result<SubmitOption> {
    let mut submit_option;
    loop {
        print_changes(changes.iter().map(|(_, content)| content.as_str()));

        submit_option = if dry_run {
            SubmitOption::Exit
        } else if submit {
            SubmitOption::Submit
        } else {
            Select::new(
                &format!("What would you like to do with {identifier} {version}?"),
                SubmitOption::iter().collect(),
            )
            .prompt()?
        };

        if submit_option == SubmitOption::Edit {
            Editor::new(changes).run()?;
        } else {
            break;
        }
    }
    Ok(submit_option)
}

#[derive(Display, EnumIter, Eq, PartialEq)]
pub enum SubmitOption {
    Submit,
    Edit,
    Exit,
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

pub fn deduplicate_display_version(
    arp_entries: Option<&mut Vec<AppsAndFeaturesEntry>>,
    package_version: &PackageVersion,
) {
    if let Some(arp) = arp_entries {
        arp.iter_mut()
            .filter(|entry| entry.display_version.as_ref() == Some(&**package_version))
            .for_each(|entry| entry.display_version = None);
    }
}
