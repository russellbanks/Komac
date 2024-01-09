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
use crate::commands::new_version::NewVersion;
use crate::commands::remove_version::RemoveVersion;
use crate::commands::token::token::{TokenArgs, TokenCommands};
use crate::commands::update_version::UpdateVersion;
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
            TokenCommands::RemoveToken(remove_token) => remove_token.run(),
            TokenCommands::UpdateToken(update_token) => update_token.run(),
        },
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
    #[command(name = "new")]
    New(Box<NewVersion>), // Boxed to store on the heap instead as New is a large struct
    #[command(name = "update")]
    Update(UpdateVersion),
    #[command(name = "remove")]
    Remove(RemoveVersion),
    Cleanup(Cleanup),
    Token(TokenArgs),
}
