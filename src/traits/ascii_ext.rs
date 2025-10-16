pub trait AsciiExt {
    fn contains_ignore_ascii_case<T>(&self, other: T) -> bool
    where
        T: AsRef<[u8]>;
}

impl AsciiExt for str {
    fn contains_ignore_ascii_case<T>(&self, other: T) -> bool
    where
        T: AsRef<[u8]>,
    {
        self.as_bytes().contains_ignore_ascii_case(other)
    }
}

impl AsciiExt for [u8] {
    fn contains_ignore_ascii_case<T>(&self, other: T) -> bool
    where
        T: AsRef<[u8]>,
    {
        let other = other.as_ref();

        if other.is_empty() {
            return true;
        }

        self.windows(other.len())
            .any(|window| window.eq_ignore_ascii_case(other))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::AsciiExt;

    #[rstest]
    #[case::basic_match("hello world", "WORLD", true)]
    #[case::reverse_case("HELLO WORLD", "world", true)]
    #[case::prefix_match("RustLang", "rust", true)]
    #[case::suffix_match("RustLang", "LANG", true)]
    #[case::no_match("hello world", "mars", false)]
    #[case::empty_other("abc", "", true)]
    #[case::empty_self("", "a", false)]
    #[case::both_empty("", "", true)]
    #[case::partial_ascii("fooBARbaz", "bar", true)]
    #[case::overlap_case("aaa", "AA", true)]
    fn contains_ignore_ascii_case_str(
        #[case] haystack: &str,
        #[case] needle: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(haystack.contains_ignore_ascii_case(needle), expected);
    }

    #[rstest]
    #[case::basic_match(b"Hello World!", b"hello", true)]
    #[case::reverse_case(b"Hello World!", b"WORLD", true)]
    #[case::no_match(b"Hello World!", b"Mars", false)]
    #[case::empty_other(b"abc", b"", true)]
    #[case::empty_self(b"", b"a", false)]
    #[case::both_empty(b"", b"", true)]
    fn contains_ignore_ascii_case_bytes(
        #[case] haystack: &[u8],
        #[case] needle: &[u8],
        #[case] expected: bool,
    ) {
        assert_eq!(haystack.contains_ignore_ascii_case(needle), expected);
    }

    #[rstest]
    #[case::string_contains_bytes("Data123", b"data", true)]
    #[case::bytes_in_string("Data123", b"123", true)]
    #[case::non_ascii_diff("Café", b"CAFE", false)]
    #[case::non_ascii_equal("Café", "Café".as_bytes(), true)]
    fn contains_ignore_ascii_case_cross_type(
        #[case] haystack: &str,
        #[case] needle: &[u8],
        #[case] expected: bool,
    ) {
        assert_eq!(haystack.contains_ignore_ascii_case(needle), expected);
    }

    #[rstest]
    #[case::accented("Café", "CAFE", false)]
    #[case::exact_match("Café", "Café", true)]
    #[case::naive_case("naïve", "NAÏVE", false)]
    fn contains_ignore_ascii_case_non_ascii(
        #[case] haystack: &str,
        #[case] needle: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(haystack.contains_ignore_ascii_case(needle), expected);
    }
}
