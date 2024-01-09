use crate::credential::get_komac_credential;
use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::style::Stylize;
use inquire::Confirm;

#[derive(Parser)]
pub struct RemoveToken {
    /// Skip the confirmation prompt to delete the token
    #[arg(short = 'y', long = "yes")]
    skip_prompt: bool,
}

impl RemoveToken {
    pub fn run(self) -> Result<()> {
        let credential = get_komac_credential()?;

        if matches!(
            credential.get_password().err(),
            Some(keyring::Error::NoEntry)
        ) {
            println!("No token stored is currently stored in the platform's secure storage");
        }

        let confirm = if self.skip_prompt {
            true
        } else {
            Confirm::new("Would you like to remove the currently stored token?").prompt()?
        };

        if confirm {
            credential.delete_password()?;
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
