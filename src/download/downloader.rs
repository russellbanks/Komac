use std::{fmt, num::NonZeroUsize};

use camino::Utf8Path;
use color_eyre::{Result, eyre::bail};
use futures_util::{StreamExt, TryFutureExt, TryStreamExt, stream};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::{Itertools, Position};
use reqwest::{
    Client,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE, GetAll, HeaderMap, HeaderValue, USER_AGENT},
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    sync::mpsc,
    try_join,
};
use winget_types::Sha256String;

use super::{Download, DownloadedFile, Downloads, PreDownload};
use crate::{
    analysis::{extensions::FileExtension, installers::msix_family::app_installer::AppInstaller},
    manifests::Url,
};

pub struct Downloader {
    client: Client,
    concurrent_downloads: NonZeroUsize,
}

impl Downloader {
    const PROGRESS_TEMPLATE: &'static str = "{msg}\n{wide_bar:.magenta/black} {decimal_bytes:.green}/{decimal_total_bytes:.green} {decimal_bytes_per_sec:.red} eta {eta:.blue}";

    const INDETERMINATE_PROGRESS_TEMPLATE: &'static str =
        "{msg}\n{spinner} {decimal_bytes:.green} {decimal_bytes_per_sec:.red} {elapsed:.blue}";

    const PROGRESS_CHARS: &'static str = "───";

    const APPLICATION: &'static str = "application";

    const OCTET_STREAM: &'static str = "octet-stream";

    /// Creates a new Downloader with a maximum number of concurrent downloads of the number of
    /// logical cores the system has.
    ///
    /// # Errors
    ///
    /// Propagates the error from [`ClientBuilder::build`] which fails if a TLS backend cannot be
    /// initialized, or the resolver cannot load the system configuration.
    ///
    /// [`ClientBuilder::build`]: reqwest::ClientBuilder::build
    #[expect(unused)]
    pub fn new() -> reqwest::Result<Self> {
        Self::new_with_concurrent(
            num_cpus::get()
                .try_into()
                .unwrap_or_else(|_| unreachable!("num_cpus::get should always returns at least 1")),
        )
    }

    /// Creates a new Downloader with a specified number of maximum concurrent downloads.
    ///
    /// # Errors
    ///
    /// Propagates the error from [`ClientBuilder::build`] which fails if a TLS backend cannot be
    /// initialized, or the resolver cannot load the system configuration.
    ///
    /// [`ClientBuilder::build`]: reqwest::ClientBuilder::build
    pub fn new_with_concurrent(concurrent_downloads: NonZeroUsize) -> reqwest::Result<Self> {
        Ok(Self {
            client: Client::builder()
                .default_headers(Self::headers())
                .referer(false)
                .build()?,
            concurrent_downloads,
        })
    }

    /// Downloads the files at the given URLs to temporary files.
    ///
    /// A file is deleted when its [`DownloadedFile`] is dropped.
    pub async fn download<I>(&self, downloads: I) -> Result<Downloads>
    where
        I: IntoIterator<Item = Url>,
    {
        let multi_progress = MultiProgress::new();

        let downloaded_files = stream::iter(downloads.into_iter().unique())
            .map(|url| {
                self.pre_fetch(&self.client, url)
                    .and_then(|download| self.fetch(download, &multi_progress))
            })
            .buffer_unordered(self.concurrent_downloads.get())
            .try_collect::<Downloads>()
            .await?;

        multi_progress.clear()?;

        Ok(downloaded_files)
    }

    /// Returns a [`HeaderMap`] of the default headers komac uses.
    ///
    /// * `user-agent`: `Microsoft-Delivery-Optimization/10.1`
    /// * `Sec-GPC`: `1`
    fn headers() -> HeaderMap {
        const MICROSOFT_DELIVERY_OPTIMIZATION: HeaderValue =
            HeaderValue::from_static("Microsoft-Delivery-Optimization/10.1");
        const SEC_GPC: &str = "Sec-GPC";

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, MICROSOFT_DELIVERY_OPTIMIZATION);
        headers.insert(SEC_GPC, HeaderValue::from(1));
        headers
    }

    fn check_content_types(
        download: &PreDownload,
        content_types: GetAll<HeaderValue>,
    ) -> Result<(), ContentTypeError> {
        if content_types.iter().all(|content_type| {
            !content_type
                .as_bytes()
                .ends_with(Self::OCTET_STREAM.as_bytes())
                && !content_type
                    .as_bytes()
                    .starts_with(Self::APPLICATION.as_bytes())
        }) {
            return Err(ContentTypeError::new(download.clone(), content_types));
        }

        Ok(())
    }

    pub async fn pre_fetch(&self, client: &Client, url: Url) -> Result<Download> {
        let mut pre_download: PreDownload = url.into();

        pre_download.convert_to_github_versioned().await?;

        pre_download.upgrade_to_https(client).await;

        loop {
            let res = client.get((***pre_download.url()).clone()).send().await?;

            if let Err(err) = res.error_for_status_ref() {
                bail!(
                    "{} returned {}",
                    err.url().unwrap().as_str(),
                    err.status().unwrap()
                )
            }

            // Check that we're downloading an application
            Self::check_content_types(&pre_download, res.headers().get_all(CONTENT_TYPE))?;

            let file_name = pre_download
                .file_name(res.url(), res.headers().get(CONTENT_DISPOSITION))
                .into_owned();

            let file_extension = if let Some(extension) = Utf8Path::new(&file_name).extension() {
                Some(extension.parse()?)
            } else {
                None
            };

            if file_extension.is_some_and(FileExtension::is_app_installer) {
                *pre_download.url_mut() = AppInstaller::fetch_main_url(res).await?.into();
                continue;
            }

            return Ok(Download::new(pre_download.into_url(), file_name, res));
        }
    }

    pub async fn fetch(
        &self,
        mut download: Download,
        multi_progress: &MultiProgress,
    ) -> Result<DownloadedFile> {
        let last_modified = download.last_modified();

        let progress_bar = match download.content_length() {
            Some(len) => ProgressBar::new(len).with_style(
                ProgressStyle::with_template(Self::PROGRESS_TEMPLATE)?
                    .progress_chars(Self::PROGRESS_CHARS),
            ),
            None => ProgressBar::no_length().with_style(ProgressStyle::with_template(
                Self::INDETERMINATE_PROGRESS_TEMPLATE,
            )?),
        };

        let progress =
            multi_progress.add(progress_bar.with_message(format!("Downloading {download}")));

        // Create a temporary file
        let temp_file = tempfile::tempfile()?;
        let file = tokio::fs::File::from_std(temp_file.try_clone()?);
        let mut buf_writer = BufWriter::new(file);

        // Create a thread for writing to the file
        let (write_sender, mut write_receiver) = mpsc::unbounded_channel::<bytes::Bytes>();
        let writer = tokio::spawn(async move {
            while let Some(chunk) = write_receiver.recv().await {
                buf_writer.write_all(&chunk).await?;
            }

            buf_writer.flush().await?;
            buf_writer.shutdown().await
        });

        // Create a thread for hashing the downloaded bytes
        let (hash_sender, hash_receiver) = crossbeam_channel::unbounded::<bytes::Bytes>();
        let hasher = tokio::task::spawn_blocking(move || {
            let mut hasher = Sha256::new();
            while let Ok(chunk) = hash_receiver.recv() {
                hasher.update(&chunk);
            }
            hasher.finalize()
        });

        let mut stream = download.response.take().unwrap().bytes_stream();

        // Download the chunks asynchronously
        while let Some(chunk) = stream.next().await.transpose()? {
            progress.inc(chunk.len() as u64);
            hash_sender.send(chunk.clone())?;
            write_sender.send(chunk)?;
        }

        drop(write_sender);
        drop(hash_sender);

        let sha_256 = match try_join!(writer, hasher)? {
            (Ok(()), sha_256) => sha_256,
            (Err(err), _) => return Err(err.into()),
        };

        progress.finish();

        Ok(DownloadedFile {
            download,
            file: temp_file,
            sha_256: Sha256String::from_digest(&sha_256),
            last_modified,
        })
    }
}

#[derive(Debug, Error)]
pub struct ContentTypeError {
    download: PreDownload,
    content_types: Vec<HeaderValue>,
}

impl ContentTypeError {
    pub fn new<D, I, C>(download: D, content_types: I) -> Self
    where
        D: Into<PreDownload>,
        I: IntoIterator<Item = C>,
        C: Into<HeaderValue>,
    {
        Self {
            download: download.into(),
            content_types: content_types.into_iter().map(C::into).collect(),
        }
    }
}

impl fmt::Display for ContentTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The content type for {} was ", self.download)?;
        for (position, content_type) in self
            .content_types
            .iter()
            .flat_map(HeaderValue::to_str)
            .with_position()
        {
            match position {
                Position { is_first: true, .. } => write!(f, "{content_type:?}")?,
                Position {
                    is_first: false,
                    is_last: false,
                } => write!(f, ", {content_type:?}")?,
                Position { is_last: true, .. } => write!(f, " and {content_type:?}")?,
            }
        }
        write!(
            f,
            " but an {application} or {octet_stream} content type was expected",
            application = Downloader::APPLICATION,
            octet_stream = Downloader::OCTET_STREAM
        )
    }
}
