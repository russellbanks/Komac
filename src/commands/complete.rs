use anstream::stdout;
use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};

use crate::Cli;

/// Outputs an autocompletion script for the given shell
#[derive(Parser)]
#[clap(visible_alias = "autocomplete")]
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
