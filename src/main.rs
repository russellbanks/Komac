extern crate core;

use crate::commands::analyse::Analyse;
use crate::commands::cleanup::Cleanup;
use crate::commands::complete::Complete;
use crate::commands::list_versions::ListVersions;
use crate::commands::new_version::NewVersion;
use crate::commands::remove_dead_versions::RemoveDeadVersions;
use crate::commands::remove_version::RemoveVersion;
use crate::commands::show_version::ShowVersion;
use crate::commands::submit::Submit;
use crate::commands::sync_fork::SyncFork;
use crate::commands::token::commands::{TokenArgs, TokenCommands};
use crate::commands::update_version::UpdateVersion;
use clap::{crate_name, Parser, Subcommand};
use color_eyre::eyre::Result;
use tracing::metadata::LevelFilter;
use tracing::Level;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod commands;
mod credential;
mod download_file;
mod editor;
mod file_analyser;
mod github;
mod hyperlink;
mod installers;
mod manifests;
mod match_installers;
mod prompts;
mod types;
mod update_state;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;

    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_target(cfg!(debug_assertions))
                .without_time(),
        )
        .with(indicatif_layer)
        .with(
            filter::Targets::new()
                .with_default(LevelFilter::DEBUG)
                .with_targets([
                    (crate_name!(), Level::TRACE),
                    ("hyper_util", Level::INFO),
                    ("html5ever", Level::INFO),
                ]),
        )
        .init();

    match Cli::parse().command {
        Commands::New(new_version) => new_version.run().await,
        Commands::Update(update_version) => update_version.run().await,
        Commands::Cleanup(cleanup) => cleanup.run().await,
        Commands::Remove(remove_version) => remove_version.run().await,
        Commands::Token(token_args) => match token_args.command {
            TokenCommands::Remove(remove_token) => remove_token.run(),
            TokenCommands::Update(update_token) => update_token.run().await,
        },
        Commands::ListVersions(list_versions) => list_versions.run().await,
        Commands::Show(show_version) => show_version.run().await,
        Commands::SyncFork(sync_fork) => sync_fork.run().await,
        Commands::Complete(complete) => {
            complete.run();
            Ok(())
        }
        Commands::Analyse(analyse) => analyse.run(),
        Commands::RemoveDeadVersions(remove_dead_versions) => remove_dead_versions.run().await,
        Commands::Submit(submit) => submit.run().await,
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None, disable_version_flag = true)]
struct Cli {
    #[arg(short = 'v', short_alias = 'V', long, action = clap::builder::ArgAction::Version)]
    version: (),
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New(Box<NewVersion>), // Boxed to store on the heap instead as New is a large struct
    Update(UpdateVersion),
    Remove(RemoveVersion),
    Cleanup(Cleanup),
    Token(TokenArgs),
    ListVersions(ListVersions),
    Show(ShowVersion),
    SyncFork(SyncFork),
    Complete(Complete),
    Analyse(Analyse),
    RemoveDeadVersions(RemoveDeadVersions),
    Submit(Submit),
}
