use crate::manifests::installer_manifest::{Installer, Scope};
use crate::url_utils::{find_architecture, VALID_FILE_EXTENSIONS};
use camino::Utf8Path;
use std::collections::HashMap;

pub fn match_installers(
    previous_installers: Vec<Installer>,
    new_installers: &[Installer],
) -> HashMap<Installer, &Installer> {
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
            Scope::find_from_url(url.as_str()).map(|scope| (url, scope))
        })
        .collect::<HashMap<_, _>>();

    previous_installers
        .into_iter()
        .map(|previous_installer| {
            let mut max_score = 0;
            let mut best_match = None;

            for new_installer in new_installers {
                let installer_url = &new_installer.installer_url;
                let mut score = 0;
                if new_installer.architecture == previous_installer.architecture {
                    score += 1;
                }
                if found_architectures.get(installer_url) == Some(&previous_installer.architecture)
                {
                    score += 1;
                }
                if new_installer.installer_type == previous_installer.installer_type {
                    score += 3;
                }
                if new_installer.scope == previous_installer.scope {
                    score += 1;
                }
                let new_extension = Utf8Path::new(new_installer.installer_url.as_str())
                    .extension()
                    .filter(|extension| VALID_FILE_EXTENSIONS.contains(extension))
                    .unwrap_or_default();
                let previous_extension = Utf8Path::new(previous_installer.installer_url.as_str())
                    .extension()
                    .filter(|extension| VALID_FILE_EXTENSIONS.contains(extension))
                    .unwrap_or_default();
                if new_extension != previous_extension {
                    score = 0;
                }

                let is_new_architecture = !found_architectures.is_empty()
                    && !found_architectures.contains_key(installer_url);
                let is_new_scope =
                    !found_scopes.is_empty() && !found_scopes.contains_key(installer_url);

                if score > max_score
                    || (score == max_score && (is_new_architecture || is_new_scope))
                    || best_match.is_none()
                {
                    max_score = score;
                    best_match = Some(new_installer);
                }
            }
            (previous_installer, best_match.unwrap())
        })
        .collect::<HashMap<_, _>>()
}

#[cfg(test)]
mod tests {
    use crate::manifests::installer_manifest::{Installer, Scope};
    use crate::match_installers::match_installers;
    use crate::types::architecture::Architecture;
    use crate::types::urls::url::Url;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn test_vscodium() {
        let installer_x86 = Installer {
            architecture: Architecture::X86,
            installer_url: Url::from_str("https://www.example.com/file-x86.exe").unwrap(),
            ..Installer::default()
        };
        let installer_user_x86 = Installer {
            scope: Some(Scope::User),
            installer_url: Url::from_str("https://www.example.com/fileUser-x86.exe").unwrap(),
            ..installer_x86.clone()
        };
        let installer_x64 = Installer {
            architecture: Architecture::X64,
            installer_url: Url::from_str("https://www.example.com/file-x64.exe").unwrap(),
            ..Installer::default()
        };
        let installer_user_x64 = Installer {
            scope: Some(Scope::User),
            installer_url: Url::from_str("https://www.example.com/fileUser-x64.exe").unwrap(),
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
            (installer_user_x86.clone(), &installer_user_x86),
            (previous_machine_x86, &installer_x86),
            (installer_user_x64.clone(), &installer_user_x64),
            (previous_machine_x64, &installer_x64),
        ]);
        assert_eq!(
            expected,
            match_installers(previous_installers, &new_installers)
        );
    }
}
