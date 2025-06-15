use color_eyre::Result;
use derive_more::Display;
use inquire::Select;
use strum::{EnumIter, IntoEnumIterator};
use winget_types::{PackageIdentifier, PackageVersion};

use crate::{
    commands::utils::environment::VHS, editor::Editor, manifests::print_changes,
    prompts::handle_inquire_error,
};

#[derive(Display, EnumIter, Eq, PartialEq)]
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
    ) -> Result<SubmitOption> {
        let mut submit_option;

        loop {
            let changes_iter = changes.iter().map(|(_, content)| content.as_str());
            if *VHS {
                print_changes(changes_iter.take(1));
            } else {
                print_changes(changes_iter);
            }

            submit_option = if dry_run {
                SubmitOption::Exit
            } else if submit {
                SubmitOption::Submit
            } else {
                Select::new(
                    &format!("What would you like to do with {identifier} {version}?"),
                    SubmitOption::iter().collect(),
                )
                .prompt()
                .map_err(handle_inquire_error)?
            };

            if submit_option == SubmitOption::Edit {
                Editor::new(changes).run()?;
            } else {
                break;
            }
        }

        Ok(submit_option)
    }
}
