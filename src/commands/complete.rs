use anstream::stdout;
use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};

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
    #[arg(value_enum)]
    shell: Shell,
}

impl Complete {
    pub fn run(self) {
        let mut command = Cli::command();
        let command_name = command.get_name().to_string();
        generate(self.shell, &mut command, command_name, &mut stdout());
    }
}
