//! Localization of user-facing text using [Project Fluent](https://projectfluent.org).
//!
//! Translations live in `i18n/<language>/komac.ftl` and are embedded into the binary. `en-US` is
//! the source of truth: every message must exist there, which [`fl!`] checks at compile time, and
//! any message missing from another language falls back to it.

use std::{env, sync::LazyLock};

use i18n_embed::{
    DesktopLanguageRequester, LanguageLoader,
    fluent::{FluentLanguageLoader, fluent_language_loader},
    unic_langid::LanguageIdentifier,
};
use rust_embed::RustEmbed;

/// Overrides the language detected from the operating system, e.g. `KOMAC_LANG=de-DE`
const LANGUAGE_ENV: &str = "KOMAC_LANG";

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

/// Starts out with only the en-US translations loaded, so messages can be looked up even if
/// [`init`] hasn't been called, as in tests
pub static LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader
        .load_fallback_language(&Localizations)
        .expect("the en-US translations are embedded in the binary");
    configure(&loader);
    loader
});

/// Loads the translations for the user's language.
///
/// This must run before the command line is parsed, as clap builds the help text then.
pub fn init() {
    // If this fails, the en-US translations stay loaded
    if i18n_embed::select(&*LOADER, &Localizations, &requested_languages()).is_ok() {
        configure(&LOADER);
    }
}

/// Returns the languages to use in order of preference, starting with the one in `KOMAC_LANG`
/// and followed by those set in the operating system
fn requested_languages() -> Vec<LanguageIdentifier> {
    env::var(LANGUAGE_ENV)
        .ok()
        .and_then(|language| language.parse::<LanguageIdentifier>().ok())
        .into_iter()
        .chain(DesktopLanguageRequester::requested_languages())
        .collect()
}

/// Configures the bundles, which are recreated each time languages are loaded
fn configure(loader: &FluentLanguageLoader) {
    loader.with_bundles_mut(|bundle| {
        // Fluent wraps arguments in Unicode isolation marks so that right-to-left text doesn't
        // reorder what surrounds it, but terminals can print them as stray characters
        bundle.set_use_isolating(false);

        // Adds `NUMBER`, used to select ordinal forms such as "1st" and "2nd". This only fails if
        // the functions were already added, which is harmless.
        let _ = bundle.add_builtins();
    });
}

/// Returns a translated message, checking at compile time that it exists in the en-US
/// translations along with any attribute and arguments given.
///
/// ```ignore
/// fl!("message-id")
/// fl!("message-id", "attribute")
/// fl!("message-id", name = value)
/// ```
macro_rules! fl {
    ($($args:tt)*) => {
        i18n_embed_fl::fl!($crate::i18n::LOADER, $($args)*)
    };
}

pub(crate) use fl;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use fluent_syntax::{ast::Entry, parser};

    use super::Localizations;

    const EN_US: &str = "en-US/komac.ftl";

    /// Returns the IDs of the messages and their attributes in a translation file, such as
    /// `field-tag` and `field-tag.help`, failing if the file has a syntax error
    fn message_ids(path: &str) -> BTreeSet<String> {
        let file = Localizations::get(path).unwrap_or_else(|| panic!("{path} is not embedded"));
        let source = str::from_utf8(&file.data).unwrap();
        let resource = parser::parse(source)
            .unwrap_or_else(|(_, errors)| panic!("{path} has syntax errors: {errors:#?}"));

        let mut ids = BTreeSet::new();
        for entry in resource.body {
            if let Entry::Message(message) = entry {
                for attribute in message.attributes {
                    ids.insert(format!("{}.{}", message.id.name, attribute.id.name));
                }
                ids.insert(message.id.name.to_owned());
            }
        }
        ids
    }

    #[test]
    fn translations_only_have_en_us_messages() {
        let en_us = message_ids(EN_US);

        for path in Localizations::iter().filter(|path| path.ends_with(".ftl")) {
            let unknown = message_ids(&path)
                .difference(&en_us)
                .cloned()
                .collect::<Vec<_>>();
            assert!(
                unknown.is_empty(),
                "{path} has messages that aren't in {EN_US}: {unknown:?}"
            );
        }
    }

    #[test]
    fn arguments_are_not_isolated() {
        assert_eq!(
            fl!("prompt-too-many-items", max = 16, count = 20),
            "There is a maximum of 16 items. There were 20 provided"
        );
    }
}
