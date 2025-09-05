use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use color_eyre::{Result, Section, eyre::eyre};

use crate::Cli;

/// Outputs an autocompletion script for the given shell. Example usage:
///
/// Bash: echo "source <(komac complete bash)" >> ~/.bashrc
/// Elvish: echo "eval (komac complete elvish | slurp)" >> ~/.elvish/rc.elv
/// Fish: echo "source (komac complete fish | psub)" >> ~/.config/fish/config.fish
/// Powershell: echo "komac complete powershell | Out-String | Invoke-Expression" >> $PROFILE
/// Zsh: echo "source <(komac complete zsh)" >> ~/.zshrc
#[derive(Parser)]
#[clap(visible_alias = "autocomplete", verbatim_doc_comment)]
pub struct Complete {
    /// Specifies the shell for which to generate the completion script.
    ///
    /// If not provided, the shell will be inferred based on the current environment.
    #[arg()]
    shell: Option<Shell>,
}

impl Complete {
    pub fn run(self) -> Result<()> {
        let Some(shell) = self.shell.or_else(Shell::from_env) else {
            return Err(
                eyre!("Unable to determine the current shell from the environment")
                    .suggestion("Specify shell explicitly"),
            );
        };

        let mut command = Cli::command();
        let command_name = command.get_name().to_owned();
        generate(shell, &mut command, command_name, &mut anstream::stdout());

        Ok(())
    }
}
