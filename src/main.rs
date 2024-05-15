extern crate core;

mod commands;
mod credential;
mod download_file;
mod editor;
mod file_analyser;
mod github;
mod manifest;
mod manifests;
mod match_installers;
mod msi;
mod msix_family;
mod prompts;
mod types;
mod update_state;
mod zip;

use crate::commands::cleanup::Cleanup;
use crate::commands::list_versions::ListVersions;
use crate::commands::new_version::NewVersion;
use crate::commands::remove_version::RemoveVersion;
use crate::commands::show_version::ShowVersion;
use crate::commands::token::commands::{TokenArgs, TokenCommands};
use crate::commands::update_version::UpdateVersion;
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;
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
}
