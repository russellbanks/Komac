use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use color_eyre::{Result, Section, eyre::eyre};

use crate::{Cli, i18n::fl};

#[derive(Parser)]
#[command(about = fl!("command-complete"), long_about = fl!("command-complete", "long"))]
#[clap(visible_alias = "autocomplete")]
pub struct Complete {
    #[arg(help = fl!("arg-complete-shell"), long_help = fl!("arg-complete-shell", "long"))]
    shell: Option<Shell>,
}

impl Complete {
    pub fn run(self) -> Result<()> {
        let Some(shell) = self.shell.or_else(Shell::from_env) else {
            return Err(eyre!(fl!("error-unknown-shell"))
                .suggestion(fl!("error-unknown-shell", "suggestion")));
        };

        let mut command = Cli::command();
        let command_name = command.get_name().to_owned();
        generate(shell, &mut command, command_name, &mut anstream::stdout());

        Ok(())
    }
}
