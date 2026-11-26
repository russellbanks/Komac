use anstream::println;
use clap::Parser;
use color_eyre::eyre::Result;
use owo_colors::OwoColorize;

use crate::{i18n::fl, prompts::text::confirm_prompt, terminal::success, token::TokenManager};

/// Remove the stored token
#[derive(Parser)]
#[clap(visible_alias = "delete")]
pub struct RemoveToken {
    /// Skip the confirmation prompt to delete the token
    #[arg(short = 'y', long = "yes")]
    skip_prompt: bool,
}

impl RemoveToken {
    pub fn run(self) -> Result<()> {
        let credential = TokenManager::credential()?;

        if matches!(
            credential.get_password().err(),
            Some(keyring_core::Error::NoEntry)
        ) {
            println!("{}", fl!("token-none-stored"));
        }

        let confirm = self.skip_prompt || confirm_prompt(&fl!("prompt-remove-token"))?;

        if confirm {
            credential.delete_credential()?;
            println!("{}", fl!("token-deleted", success = success()));
        } else {
            println!("{}", fl!("token-not-deleted").cyan());
        }

        Ok(())
    }
}
