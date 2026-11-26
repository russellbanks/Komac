use anstream::println;
use clap::Parser;
use color_eyre::eyre::Result;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

use crate::{
    i18n::fl,
    terminal::success,
    token::{TokenManager, default_headers},
};

#[derive(Parser)]
#[command(about = fl!("command-token-update"))]
#[clap(visible_aliases = ["new", "add"])]
pub struct UpdateToken {
    #[arg(short, long, help = fl!("arg-token-update-token"))]
    token: Option<SecretString>,
}

impl UpdateToken {
    pub async fn run(self) -> Result<()> {
        let credential = TokenManager::credential()?;

        let client = Client::builder()
            .default_headers(default_headers(None))
            .build()?;

        let token = match self.token {
            Some(token) => {
                TokenManager::validate(&client, token.expose_secret()).await?;
                token
            }
            None => TokenManager::prompt()
                .client(&client)
                .message(&fl!("prompt-new-token"))
                .call()?,
        };

        credential.set_password(token.expose_secret())?;

        println!("{}", fl!("token-stored", success = success()));

        Ok(())
    }
}
