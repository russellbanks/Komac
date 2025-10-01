use winget_types::installer::Installer;

pub fn match_installers<I>(previous_installers: &[Installer], new_installers: I) -> Vec<Installer>
where
    I: IntoIterator<Item = Installer>,
{
    let mut installers = Vec::new();

    for new_installer in new_installers {
        if let Some(prev) = previous_installers.iter().find(|previous_installer| {
            previous_installer.architecture == new_installer.architecture
                && previous_installer.r#type == new_installer.r#type
                && (new_installer.scope.is_some()
                    && new_installer.scope == previous_installer.scope)
        }) {
            installers.push(new_installer.merge_with(prev.clone()));
        } else {
            installers.push(new_installer);
        }
    }

    installers
}
