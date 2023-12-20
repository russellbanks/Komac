use crate::manifests::installer_manifest::Installer;
use crate::url_utils::{find_architecture, find_scope};
use std::collections::HashMap;

pub fn match_installers(
    previous_installers: &[Installer],
    new_installers: &[Installer],
) -> HashMap<Installer, Installer> {
    let found_architectures = new_installers
        .iter()
        .filter_map(|installer| {
            let url = &installer.installer_url;
            find_architecture(url.as_str()).map(|architecture| (url, architecture))
        })
        .collect::<HashMap<_, _>>();

    let found_scopes = new_installers
        .iter()
        .filter_map(|installer| {
            let url = &installer.installer_url;
            find_scope(url.as_str()).map(|scope| (url, scope))
        })
        .collect::<HashMap<_, _>>();

    previous_installers
        .iter()
        .map(|previous_installer| {
            let mut max_score = 0;
            let mut best_match = None;

            for new_installer in new_installers {
                let installer_url = &new_installer.installer_url;
                let mut score = 0;
                if new_installer.architecture == previous_installer.architecture {
                    score += 1
                }
                if found_architectures.get(installer_url) == Some(&previous_installer.architecture)
                {
                    score += 1
                }
                if new_installer.installer_type == previous_installer.installer_type {
                    score += 1
                }
                if new_installer.scope == previous_installer.scope {
                    score += 1
                }

                let is_new_architecture = !found_architectures
                    .values()
                    .any(|_| found_architectures.get(installer_url).is_some());
                let is_new_scope = !found_scopes.is_empty()
                    && !found_scopes
                        .values()
                        .any(|_| found_scopes.get(installer_url).is_some());

                if score > max_score
                    || (score == max_score && (is_new_architecture || is_new_scope))
                    || best_match.is_none()
                {
                    max_score = score;
                    best_match = Some(new_installer);
                }
            }
            (
                previous_installer.to_owned(),
                best_match.unwrap().to_owned(),
            )
        })
        .collect::<HashMap<_, _>>()
}

#[cfg(test)]
mod tests {
    use crate::manifests::installer_manifest::{Architecture, Installer, Scope};
    use crate::match_installers::match_installers;
    use std::collections::HashMap;
    use url::Url;

    #[test]
    fn test_vscodium() {
        let installer_x86 = Installer {
            architecture: Architecture::X86,
            installer_url: Url::parse("https://www.example.com/file-x86.exe").unwrap(),
            ..Installer::default()
        };
        let installer_user_x86 = Installer {
            scope: Some(Scope::User),
            installer_url: Url::parse("https://www.example.com/fileUser-x86.exe").unwrap(),
            ..installer_x86.clone()
        };
        let installer_x64 = Installer {
            architecture: Architecture::X64,
            installer_url: Url::parse("https://www.example.com/file-x64.exe").unwrap(),
            ..Installer::default()
        };
        let installer_user_x64 = Installer {
            scope: Some(Scope::User),
            installer_url: Url::parse("https://www.example.com/fileUser-x64.exe").unwrap(),
            ..installer_x64.clone()
        };
        let previous_machine_x86 = Installer {
            scope: Some(Scope::Machine),
            ..installer_x86.clone()
        };
        let previous_machine_x64 = Installer {
            scope: Some(Scope::Machine),
            ..installer_x64.clone()
        };
        let previous_installers = vec![
            installer_user_x86.clone(),
            previous_machine_x86.clone(),
            installer_user_x64.clone(),
            previous_machine_x64.clone(),
        ];
        let new_installers = vec![
            installer_user_x86.clone(),
            installer_x86.clone(),
            installer_user_x64.clone(),
            installer_x64.clone(),
        ];
        let expected = HashMap::from([
            (installer_user_x86.clone(), installer_user_x86.clone()),
            (previous_machine_x86.clone(), installer_x86.clone()),
            (installer_user_x64.clone(), installer_user_x64.clone()),
            (previous_machine_x64.clone(), installer_x64.clone()),
        ]);
        assert_eq!(
            expected,
            match_installers(&previous_installers, &new_installers)
        );
    }
}
