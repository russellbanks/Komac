mod hyperlink;

pub use hyperlink::{Hyperlinkable, SUPPORTS_HYPERLINKS};
use owo_colors::OwoColorize;

use crate::i18n::fl;

/// Returns "Successfully" highlighted in green, for messages that report success
pub fn success() -> String {
    fl!("success").green().to_string()
}
