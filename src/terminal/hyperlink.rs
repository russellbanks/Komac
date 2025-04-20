use std::{fmt, sync::LazyLock};

static SUPPORTS_HYPERLINKS: LazyLock<bool> =
    LazyLock::new(supports_hyperlinks::supports_hyperlinks);

pub struct Hyperlink<'text, D: fmt::Display, U: fmt::Display> {
    text: &'text D,
    url: U,
}

impl<D: fmt::Display, U: fmt::Display> fmt::Display for Hyperlink<'_, D, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *SUPPORTS_HYPERLINKS {
            write!(f, "\x1B]8;;{}\x1B\\{}\x1B]8;;\x1B\\", self.url, self.text)
        } else {
            write!(f, "{}", self.text)
        }
    }
}

pub trait Hyperlinkable<D: fmt::Display, U: fmt::Display> {
    fn hyperlink(&self, url: U) -> Hyperlink<D, U>;
}

impl<D: fmt::Display, U: fmt::Display> Hyperlinkable<D, U> for D {
    fn hyperlink(&self, url: U) -> Hyperlink<D, U> {
        Hyperlink { text: self, url }
    }
}
