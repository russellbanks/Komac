use std::fmt;

use color_eyre::Result;
use inquire::Select;
use winget_types::{PackageIdentifier, PackageVersion};

use crate::{
    editor::Editor,
    github::utils::pull_request::{Change, Changes},
    i18n::fl,
    manifests::print_changes,
    prompts::handle_inquire_error,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SubmitOption {
    Submit,
    Edit,
    Exit,
}

impl SubmitOption {
    pub fn prompt(
        changes: &mut Changes,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
        submit: bool,
        exit: bool,
    ) -> Result<Self> {
        let mut submit_option;

        loop {
            print_changes(changes.iter().map(Change::manifest));

            submit_option = if submit {
                Self::Submit
            } else if exit {
                Self::Exit
            } else {
                Select::new(
                    &fl!(
                        "prompt-submit-option",
                        identifier = identifier.as_str(),
                        version = version.to_string()
                    ),
                    Self::all().into(),
                )
                .prompt()
                .map_err(handle_inquire_error)?
            };

            if submit_option.is_edit() {
                Editor::new(changes).run()?;
            } else {
                break;
            }
        }

        Ok(submit_option)
    }

    /// Returns `true` if the submit option is submit.
    #[expect(unused)]
    #[inline]
    pub const fn is_submit(self) -> bool {
        matches!(self, Self::Submit)
    }

    /// Returns `true` if the submit option is edit.
    #[inline]
    pub const fn is_edit(self) -> bool {
        matches!(self, Self::Edit)
    }

    /// Returns `true` if the submit option is exit.
    #[inline]
    pub const fn is_exit(self) -> bool {
        matches!(self, Self::Exit)
    }

    /// Returns an array of all the submit options.
    #[inline]
    pub const fn all() -> [Self; 3] {
        [Self::Submit, Self::Edit, Self::Exit]
    }

    /// Returns the translated label shown in the prompt
    pub fn label(self) -> String {
        match self {
            Self::Submit => fl!("prompt-submit-option", "submit"),
            Self::Edit => fl!("prompt-submit-option", "edit"),
            Self::Exit => fl!("prompt-submit-option", "exit"),
        }
    }
}

impl fmt::Display for SubmitOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label())
    }
}
