mod commands;
mod credential;
mod download_file;
mod file_analyser;
mod github;
mod graphql;
mod manifest;
mod manifests;
mod match_installers;
mod msi;
mod msix_family;
mod prompts;
mod types;
mod update_state;
mod url_utils;
mod zip;

use crate::commands::cleanup::Cleanup;
use crate::commands::new_version::New;
use crate::commands::remove_version::Remove;
use crate::commands::update::Update;
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;
    match Cli::parse().command {
        Commands::New(new_version) => new_version.run().await,
        Commands::Update(update) => update.run().await,
        Commands::Cleanup(cleanup) => cleanup.run().await,
        Commands::Remove(remove_version) => remove_version.run().await,
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New(Box<New>), // Boxed to store on the heap instead as New is a large struct
    Update(Update),
    Cleanup(Cleanup),
    Remove(Remove),
}
