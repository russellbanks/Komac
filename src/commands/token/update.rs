use crate::credential::{get_default_headers, get_komac_credential, token_prompt, validate_token};
use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::style::Stylize;
use reqwest::Client;

#[derive(Parser)]
pub struct UpdateToken {
    /// The new token to store
    #[arg(short, long)]
    token: Option<String>,
}

impl UpdateToken {
    pub async fn run(self) -> Result<()> {
        let credential = get_komac_credential()?;

        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;

        let token = match self.token {
            Some(token) => validate_token(&client, &token).await.map(|()| token)?,
            None => token_prompt(client, Some("Please enter the new token to set"))?,
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
