use std::{borrow::Cow, fmt::Write, io, path::Path, slice, vec};

use clap::{crate_name, crate_version};
use const_format::concatc;
use futures_util::{StreamExt, TryStreamExt, stream};
use serde::Serialize;
use tokio::{fs, fs::File, io::AsyncWriteExt};
use winget_types::Manifest;

pub struct Changes(Vec<Change>);

impl Changes {
    pub fn new<I>(changes: I) -> Self
    where
        I: IntoIterator<Item = Change>,
    {
        Self(changes.into_iter().collect())
    }

    pub async fn write_to(&self, directory: &Path) -> io::Result<()> {
        // Create parent directories recursively
        fs::create_dir_all(directory).await?;

        stream::iter(self.0.iter())
            .map(|change| async move { change.write_to(directory).await })
            .buffer_unordered(2)
            .try_collect()
            .await
    }

    /// Returns an iterator over the changes.
    pub fn iter(&self) -> slice::Iter<'_, Change> {
        self.0.iter()
    }

    /// Returns an iterator that allows modifying each value.
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, Change> {
        self.0.iter_mut()
    }
}

impl IntoIterator for Changes {
    type Item = Change;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct Change {
    pub(crate) path: String,
    pub(crate) manifest: String,
}

impl Change {
    pub fn new<P, M>(path: P, manifest: &M, created_with: Option<&str>) -> Self
    where
        P: Into<String>,
        M: Manifest + Serialize,
    {
        let mut result = String::from("# Created with ");
        if let Some(created_with_tool) = created_with {
            let _ = write!(result, "{created_with_tool} using ");
        }
        let _ = writeln!(result, "{} v{}", crate_name!(), crate_version!());
        let _ = writeln!(result, "# yaml-language-server: $schema={}", M::SCHEMA);
        let _ = writeln!(result);
        let _ = write!(result, "{}", serde_yaml::to_string(manifest).unwrap());

        Self {
            path: path.into(),
            manifest: convert_to_crlf(&result).into_owned(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn path(&self) -> &str {
        self.path.as_str()
    }

    #[must_use]
    #[inline]
    pub const fn manifest(&self) -> &str {
        self.manifest.as_str()
    }

    pub async fn write_to(&self, directory: &Path) -> io::Result<()> {
        if let Some(file_name) = Path::new(self.path()).file_name() {
            let mut file = File::create(directory.join(file_name)).await?;
            file.write_all(self.manifest().as_bytes()).await?;
        }

        Ok(())
    }
}

fn convert_to_crlf(input: &str) -> Cow<'_, str> {
    const CR: char = '\r';
    const LF: char = '\n';
    const CRLF: &str = concatc!(CR, LF);

    let mut buffer = None;
    let mut position = 0;
    let mut chars = input.char_indices().peekable();

    while let Some((index, char)) = chars.next() {
        match char {
            CR => {
                let buf = buffer.get_or_insert_with(|| String::with_capacity(input.len()));

                // Copy text before CR
                buf.push_str(&input[position..index]);

                // Check for CR+LF
                if let Some(&(_, LF)) = chars.peek() {
                    // Skip the LF as we'll add CRLF
                    chars.next();
                }

                buf.push_str(CRLF);

                position = chars
                    .peek()
                    .map_or(input.len(), |&(next_index, _)| next_index);
            }
            LF => {
                // Convert LF
                let buf = buffer.get_or_insert_with(|| String::with_capacity(input.len()));
                buf.push_str(&input[position..index]);
                buf.push_str(CRLF);
                position = index + LF.len_utf8();
            }
            _ => {}
        }
    }

    buffer.map_or(Cow::Borrowed(input), |mut buf| {
        buf.push_str(&input[position..]);
        Cow::Owned(buf)
    })
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::convert_to_crlf;

    #[test]
    fn preserves_valid_crlf() {
        assert_eq!(
            convert_to_crlf("Valid\r\nLine"),
            Cow::Borrowed("Valid\r\nLine")
        );
    }

    #[test]
    fn converts_lf_to_crlf() {
        assert_eq!(
            convert_to_crlf("Unix\nLine"),
            Cow::Owned::<str>("Unix\r\nLine".into())
        );
    }

    #[test]
    fn converts_lone_cr_to_crlf() {
        assert_eq!(
            convert_to_crlf("Old\rMac"),
            Cow::Owned::<str>("Old\r\nMac".into())
        );
    }

    #[test]
    fn mixed_conversions() {
        assert_eq!(
            convert_to_crlf("Mix\r\n\n\rEnd"),
            Cow::Owned::<str>("Mix\r\n\r\n\r\nEnd".into())
        );
    }

    #[test]
    fn no_changes_needed() {
        assert_eq!(convert_to_crlf("No changes"), Cow::Borrowed("No changes"));
    }

    #[test]
    fn empty_string() {
        assert_eq!(convert_to_crlf(""), Cow::Borrowed(""));
    }

    #[test]
    fn edge_cases() {
        assert_eq!(convert_to_crlf("\r"), "\r\n");
        assert_eq!(convert_to_crlf("\n"), "\r\n");
        assert_eq!(convert_to_crlf("\r\n"), "\r\n");
        assert_eq!(convert_to_crlf("a\rb\nc\r\nd"), "a\r\nb\r\nc\r\nd");
    }
}
