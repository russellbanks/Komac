use anstream::println;
use clap::Parser;
use color_eyre::eyre::Result;
use owo_colors::OwoColorize;

use crate::{prompts::text::confirm_prompt, token::TokenManager};

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
            Some(keyring::Error::NoEntry)
        ) {
            println!("No token stored is currently stored in the platform's secure storage");
        }

        let confirm = self.skip_prompt
            || confirm_prompt("Would you like to remove the currently stored token?")?;

        if confirm {
            credential.delete_credential()?;
            println!(
                "{} deleted the stored token from the platform's secure storage",
                "Successfully".green()
            );
        } else {
            println!("{}", "No token was deleted".cyan());
        }

        Ok(())
    }
}
