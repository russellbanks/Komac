use clap::{Args, Subcommand};

use crate::commands::token::{remove::RemoveToken, update::UpdateToken};

#[derive(Args)]
pub struct TokenArgs {
    #[command(subcommand)]
    pub command: TokenCommands,
}
#[derive(Subcommand)]
pub enum TokenCommands {
    Update(UpdateToken),
    Remove(RemoveToken),
}
