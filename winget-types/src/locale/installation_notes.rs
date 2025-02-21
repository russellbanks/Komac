use crate::winget_value;

winget_value!(InstallationNotes, 1, 10000);

#[cfg(test)]
mod tests {
    use crate::{locale::installation_notes::InstallationNotes, shared::value::ValueError};

    #[test]
    fn valid_installation_notes() {
        assert!(InstallationNotes::new("Be careful when using this application").is_ok());
    }

    #[test]
    fn installation_notes_max_length() {
        let installation_notes = "🦀".repeat(InstallationNotes::MAX_CHAR_LENGTH);

        // Ensure that it's character length that's being checked and not byte or UTF-16 length
        assert!(installation_notes.len() > InstallationNotes::MAX_CHAR_LENGTH);
        assert!(installation_notes.encode_utf16().count() > InstallationNotes::MAX_CHAR_LENGTH);
        assert_eq!(
            installation_notes.chars().count(),
            InstallationNotes::MAX_CHAR_LENGTH
        );
        assert!(InstallationNotes::new(installation_notes).is_ok());
    }

    #[test]
    fn installation_notes_too_long() {
        let installation_notes = "a".repeat(InstallationNotes::MAX_CHAR_LENGTH + 1);

        assert_eq!(
            InstallationNotes::new(installation_notes).err().unwrap(),
            ValueError::TooLong
        );
    }

    #[test]
    fn empty_installation_notes() {
        assert_eq!(
            InstallationNotes::new("").err().unwrap(),
            ValueError::TooShort
        );
    }
}
