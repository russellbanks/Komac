use crate::credential::{get_default_headers, get_komac_credential, token_prompt, validate_token};
use anstream::println;
use clap::Parser;
use color_eyre::eyre::Result;
use owo_colors::OwoColorize;
use reqwest::Client;

/// 更新存储的令牌
#[derive(Parser)]
#[clap(visible_alias = "new")]
pub struct UpdateToken {
    /// 要存储的新令牌
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
            None => token_prompt(client, Some("请输入要设置的新令牌"))?,
        };

        if credential.set_password(&token).is_ok() {
            println!(
                "{} 已将令牌存储在平台的安全存储中",
                "成功".green()
            );
        }

        Ok(())
    }
}
