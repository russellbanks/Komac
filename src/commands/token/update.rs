use anstream::println;
use clap::Parser;
use color_eyre::eyre::Result;
use owo_colors::OwoColorize;
use reqwest::Client;

use crate::token::{TokenManager, default_headers};

/// Update the stored token
#[derive(Parser)]
#[clap(visible_aliases = ["new", "add"])]
pub struct UpdateToken {
    /// The new token to store
    #[arg(short, long)]
    token: Option<String>,
}

impl UpdateToken {
    pub async fn run(self) -> Result<()> {
        let credential = TokenManager::credential()?;

        let client = Client::builder()
            .default_headers(default_headers(None))
            .build()?;

        let token = match self.token {
            Some(token) => {
                TokenManager::validate(&client, &token).await?;
                token
            }
            None => TokenManager::prompt()
                .client(&client)
                .message("Please enter the new token to set")
                .call()?,
        };

        if credential.set_password(&token).is_ok() {
            println!(
                "{} stored token in platform's secure storage",
                "Successfully".green()
            );
        }

        Ok(())
    }
}
