use std::{
    collections::HashMap,
    io::{Read, Seek},
    mem,
};

use color_eyre::Result;
use futures_util::{StreamExt, TryStreamExt, stream};
use tracing::debug;
use winget_types::{installer::Architecture, url::DecodedUrl};

use super::DownloadedFile;
use crate::analysis::Analyzer;

#[derive(Default)]
pub struct Downloads(Vec<DownloadedFile>);

impl Downloads {
    /// Creates a new [`Downloads`] from an iterator of [`DownloadedFile`].
    #[expect(unused)]
    pub fn new<I>(downloads: I) -> Self
    where
        I: IntoIterator<Item = DownloadedFile>,
    {
        Self(downloads.into_iter().collect())
    }

    pub async fn analyze(&mut self) -> Result<HashMap<DecodedUrl, Analyzer<'_, impl Read + Seek>>> {
        stream::iter(self.0.iter_mut().map(
            |DownloadedFile {
                 file,
                 url,
                 sha_256,
                 file_name,
                 last_modified,
                 ..
             }| async move {
                let mut file_analyzer = Analyzer::new(file, file_name)?;
                let architecture = url
                    .override_architecture()
                    .or_else(|| Architecture::from_url(url.as_str()));
                for installer in &mut file_analyzer.installers {
                    if let Some(architecture) = architecture {
                        installer.architecture = architecture;
                    }
                    debug!("{url}: {architecture:?}");
                    installer.url = url.inner().clone();
                    installer.sha_256 = sha_256.clone();
                    installer.release_date = *last_modified;
                }
                file_analyzer.file_name = mem::take(file_name);
                Ok((mem::take(url.inner_mut()), file_analyzer))
            },
        ))
        .buffer_unordered(num_cpus::get())
        .try_collect::<HashMap<_, _>>()
        .await
    }
}

impl Extend<DownloadedFile> for Downloads {
    fn extend<T: IntoIterator<Item = DownloadedFile>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}
