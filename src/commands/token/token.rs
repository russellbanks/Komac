use crate::commands::token::remove::RemoveToken;
use crate::commands::token::update::UpdateToken;
use clap::{Args, Subcommand};

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
