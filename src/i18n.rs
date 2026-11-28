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
    use std::collections::{BTreeMap, BTreeSet};

    use fluent_syntax::{
        ast::{Entry, Expression, InlineExpression, Pattern, PatternElement},
        parser,
    };
    use i18n_embed::{
        fluent::{FluentLanguageLoader, fluent_language_loader},
        unic_langid::LanguageIdentifier,
    };
    use rstest::rstest;

    use super::Localizations;

    const EN_US: &str = "en-US/komac.ftl";

    /// Returns the translation files, such as `zh-CN/komac.ftl`, other than en-US
    fn translations() -> impl Iterator<Item = String> {
        Localizations::iter()
            .filter(|path| path.ends_with(".ftl") && path != EN_US)
            .map(Into::into)
    }

    /// Returns the messages and their attributes in a translation file, such as `field-tag` and
    /// `field-tag.help`, with the arguments each one uses, failing if the file has a syntax error
    fn messages(path: &str) -> BTreeMap<String, BTreeSet<String>> {
        let file = Localizations::get(path).unwrap_or_else(|| panic!("{path} is not embedded"));
        let source = str::from_utf8(&file.data).unwrap();
        let resource = parser::parse(source)
            .unwrap_or_else(|(_, errors)| panic!("{path} has syntax errors: {errors:#?}"));

        let mut messages = BTreeMap::new();
        for entry in resource.body {
            if let Entry::Message(message) = entry {
                let id = message.id.name;
                for attribute in &message.attributes {
                    messages.insert(
                        format!("{id}.{}", attribute.id.name),
                        arguments(&attribute.value),
                    );
                }
                if let Some(value) = &message.value {
                    messages.insert(id.to_owned(), arguments(value));
                }
            }
        }
        messages
    }

    /// Returns the names of the arguments a pattern uses, including those in selectors
    fn arguments(pattern: &Pattern<&str>) -> BTreeSet<String> {
        fn pattern_arguments(pattern: &Pattern<&str>, names: &mut BTreeSet<String>) {
            for element in &pattern.elements {
                if let PatternElement::Placeable { expression } = element {
                    expression_arguments(expression, names);
                }
            }
        }

        fn expression_arguments(expression: &Expression<&str>, names: &mut BTreeSet<String>) {
            match expression {
                Expression::Select { selector, variants } => {
                    inline_arguments(selector, names);
                    for variant in variants {
                        pattern_arguments(&variant.value, names);
                    }
                }
                Expression::Inline(inline) => inline_arguments(inline, names),
            }
        }

        fn inline_arguments(inline: &InlineExpression<&str>, names: &mut BTreeSet<String>) {
            match inline {
                InlineExpression::VariableReference { id } => {
                    names.insert(id.name.to_owned());
                }
                InlineExpression::FunctionReference { arguments, .. } => {
                    for argument in &arguments.positional {
                        inline_arguments(argument, names);
                    }
                    for argument in &arguments.named {
                        inline_arguments(&argument.value, names);
                    }
                }
                InlineExpression::Placeable { expression } => {
                    expression_arguments(expression, names);
                }
                _ => {}
            }
        }

        let mut names = BTreeSet::new();
        pattern_arguments(pattern, &mut names);
        names
    }

    #[test]
    fn translations_only_have_en_us_messages() {
        let en_us = messages(EN_US);

        for path in translations() {
            let unknown = messages(&path)
                .into_keys()
                .filter(|id| !en_us.contains_key(id))
                .collect::<Vec<_>>();
            assert!(
                unknown.is_empty(),
                "{path} has messages that aren't in {EN_US}: {unknown:?}"
            );
        }
    }

    /// A translation that leaves out an argument would drop information such as a version, and
    /// one that adds an argument would show a Fluent error, as Komac doesn't pass it
    #[test]
    fn translations_use_en_us_arguments() {
        let en_us = messages(EN_US);

        for path in translations() {
            for (id, arguments) in messages(&path) {
                let expected = &en_us[&id];
                assert_eq!(
                    &arguments, expected,
                    "{path}: {id} uses {arguments:?} but {EN_US} uses {expected:?}"
                );
            }
        }
    }

    /// Simplified Chinese is chosen however the operating system names it: Windows uses `zh-CN`,
    /// macOS `zh-Hans-CN` and Linux `zh_CN`. Without a Traditional Chinese translation, Fluent
    /// matches on the language alone, so Traditional Chinese systems get Simplified Chinese
    /// rather than English.
    #[rstest]
    #[case("zh-CN", "zh-CN")]
    #[case("zh-Hans-CN", "zh-CN")]
    #[case("zh-Hans", "zh-CN")]
    #[case("zh", "zh-CN")]
    #[case("zh-SG", "zh-CN")]
    #[case("zh-TW", "zh-CN")]
    #[case("zh-HK", "zh-CN")]
    #[case("en-GB", "en-US")]
    #[case("de-DE", "en-US")]
    fn language_negotiation(#[case] requested: &str, #[case] expected: &str) {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        let requested = [requested.parse::<LanguageIdentifier>().unwrap()];

        let selected = i18n_embed::select(&loader, &Localizations, &requested).unwrap();

        assert_eq!(
            selected.first().map(ToString::to_string).as_deref(),
            Some(expected)
        );
    }

    #[rstest]
    #[case(1, "1st")]
    #[case(2, "2nd")]
    #[case(3, "3rd")]
    #[case(4, "4th")]
    #[case(11, "11th")]
    #[case(12, "12th")]
    #[case(13, "13th")]
    #[case(21, "21st")]
    #[case(102, "102nd")]
    fn ordinal(#[case] position: usize, #[case] expected: &str) {
        assert_eq!(
            fl!("prompt-installer-url", position = position),
            format!("{expected} Installer URL")
        );
    }

    #[test]
    fn arguments_are_not_isolated() {
        assert_eq!(
            fl!("prompt-portable-exe", file = "setup.exe"),
            "Is setup.exe a portable exe?"
        );
    }
}
