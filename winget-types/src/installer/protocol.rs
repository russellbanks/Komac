use crate::winget_value;

winget_value!(Protocol, 1, 2048);

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::installer::protocol::Protocol;

    #[test]
    fn serialize_protocol() {
        assert_eq!(
            serde_yaml::to_string(&Protocol::new("ftp").unwrap()).unwrap(),
            indoc! {"
                ftp
            "}
        );
    }
}
