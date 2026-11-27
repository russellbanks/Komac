use anstream::println;
use clap::Parser;
use color_eyre::eyre::Result;
use owo_colors::OwoColorize;

use crate::{i18n::fl, prompts::text::confirm_prompt, terminal::success, token::TokenManager};

#[derive(Parser)]
#[command(about = fl!("command-token-remove"))]
#[clap(visible_alias = "delete")]
pub struct RemoveToken {
    #[arg(short = 'y', long = "yes", help = fl!("arg-token-remove-skip-prompt"))]
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
