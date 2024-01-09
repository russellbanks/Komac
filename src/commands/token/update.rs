use crate::credential::get_komac_credential;
use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::style::Stylize;
use inquire::Text;

#[derive(Parser)]
pub struct UpdateToken {
    /// The new token to store
    #[arg(short, long)]
    token: Option<String>,
}

impl UpdateToken {
    pub fn run(self) -> Result<()> {
        let credential = get_komac_credential()?;

        let token = match self.token {
            Some(token) => token,
            None => Text::new("Please enter the new token to set").prompt()?,
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
