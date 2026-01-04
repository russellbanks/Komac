use camino::{Utf8Component, Utf8Path, Utf8PathBuf};

pub trait NormalizePath {
    /// Normalize a path without performing I/O.
    ///
    /// All redundant separator and up-level references are collapsed.
    ///
    /// However, this does not resolve links.
    fn normalize(&self) -> Utf8PathBuf;
}

pub trait LowercaseExtension {
    fn lowercase_extension(&self) -> Utf8PathBuf;
}

impl LowercaseExtension for Utf8PathBuf {
    fn lowercase_extension(&self) -> Utf8PathBuf {
        self.with_extension(self.extension().unwrap_or_default().to_ascii_lowercase())
    }
}

impl NormalizePath for Utf8Path {
    fn normalize(&self) -> Utf8PathBuf {
        let mut components = self.components().peekable();
        let mut ret = if let Some(c @ Utf8Component::Prefix(..)) = components.peek() {
            let buf = Utf8PathBuf::from(c.as_str());
            components.next();
            buf
        } else {
            Utf8PathBuf::new()
        };

        for component in components {
            match component {
                Utf8Component::Prefix(..) => unreachable!(),
                Utf8Component::RootDir => {
                    ret.push(component.as_str());
                }
                Utf8Component::CurDir => {}
                Utf8Component::ParentDir => {
                    ret.pop();
                }
                Utf8Component::Normal(c) => {
                    ret.push(c);
                }
            }
        }

        ret
    }
}
