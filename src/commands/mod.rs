pub mod analyze;
pub mod cleanup;
pub mod complete;
pub mod list_versions;
pub mod new_version;
pub mod remove_dead_versions;
pub mod remove_version;
pub mod show_version;
pub mod submit;
pub mod sync_fork;
pub mod token;
pub mod update_version;
pub mod utils;

use analyze::Analyze;
use clap::Subcommand;
use cleanup::Cleanup;
use complete::Complete;
use list_versions::ListVersions;
use new_version::NewVersion;
use remove_dead_versions::RemoveDeadVersions;
use remove_version::RemoveVersion;
use show_version::ShowVersion;
use submit::Submit;
use sync_fork::SyncFork;
use token::commands::{TokenArgs, TokenCommands};
use update_version::UpdateVersion;

#[derive(Subcommand)]
pub enum Commands {
    New(Box<NewVersion>),       // Comparatively large so boxed to store on the heap
    Update(Box<UpdateVersion>), // Comparatively large so boxed to store on the heap
    Remove(RemoveVersion),
    Cleanup(Cleanup),
    Token(TokenArgs),
    List(ListVersions),
    Show(ShowVersion),
    Sync(SyncFork),
    Complete(Complete),
    Analyze(Analyze),
    RemoveDeadVersions(RemoveDeadVersions),
    Submit(Submit),
}

impl Commands {
    pub async fn run(self) -> color_eyre::Result<()> {
        match self {
            Self::New(new_version) => new_version.run().await,
            Self::Update(update_version) => update_version.run().await,
            Self::Cleanup(cleanup) => cleanup.run().await,
            Self::Remove(remove_version) => remove_version.run().await,
            Self::Token(token_args) => match token_args.command {
                TokenCommands::Remove(remove_token) => remove_token.run(),
                TokenCommands::Update(update_token) => update_token.run().await,
            },
            Self::List(list_versions) => list_versions.run().await,
            Self::Show(show_version) => show_version.run().await,
            Self::Sync(sync_fork) => sync_fork.run().await,
            Self::Complete(complete) => complete.run(),
            Self::Analyze(analyse) => analyse.run(),
            Self::RemoveDeadVersions(remove_dead_versions) => remove_dead_versions.run().await,
            Self::Submit(submit) => submit.run().await,
        }
    }
}
