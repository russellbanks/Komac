use std::fmt;

use color_eyre::Result;
use inquire::Select;
use winget_types::{PackageIdentifier, PackageVersion};

use crate::{
    commands::utils::environment::VHS, editor::Editor, manifests::print_changes,
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
        changes: &mut [(String, String)],
        identifier: &PackageIdentifier,
        version: &PackageVersion,
        submit: bool,
        dry_run: bool,
    ) -> Result<Self> {
        let mut submit_option;

        loop {
            let changes_iter = changes.iter().map(|(_, content)| content.as_str());
            if *VHS {
                print_changes(changes_iter.take(1));
            } else {
                print_changes(changes_iter);
            }

            submit_option = if dry_run {
                Self::Exit
            } else if submit {
                Self::Submit
            } else {
                Select::new(
                    &format!("What would you like to do with {identifier} {version}?"),
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

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Submit => "Submit",
            Self::Edit => "Edit",
            Self::Exit => "Exit",
        }
    }
}

impl fmt::Display for SubmitOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
