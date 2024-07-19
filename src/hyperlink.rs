pub trait Hyperlink<S: AsRef<str>, T: AsRef<str>> {
    fn hyperlink(&self, url: T) -> String;
}

impl<S: AsRef<str>, T: AsRef<str>> Hyperlink<S, T> for S {
    fn hyperlink(&self, url: T) -> String {
        format!(
            "\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
            url.as_ref(),
            self.as_ref()
        )
    }
}
