use std::num::NonZeroUsize;

use chrono::DateTime;
use color_eyre::{Result, eyre::bail};
use futures_util::{StreamExt, TryStreamExt, stream};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools;
use memmap2::Mmap;
use reqwest::{
    Client,
    header::{
        CONTENT_DISPOSITION, CONTENT_TYPE, DNT, HeaderMap, HeaderValue, LAST_MODIFIED, USER_AGENT,
    },
};
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    sync::mpsc,
    try_join,
};
use winget_types::Sha256String;

use super::{Download, DownloadedFile};

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

    pub async fn download<I, D>(&self, downloads: I) -> Result<Vec<DownloadedFile>>
    where
        I: IntoIterator<Item = D>,
        D: Into<Download>,
    {
        let multi_progress = MultiProgress::new();

        let downloaded_files = stream::iter(downloads.into_iter().map(D::into).unique())
            .map(|download| self.fetch(&self.client, download, &multi_progress))
            .buffer_unordered(self.concurrent_downloads.get())
            .try_collect::<Vec<_>>()
            .await?;

        multi_progress.clear()?;

        Ok(downloaded_files)
    }

    fn headers() -> HeaderMap {
        const MICROSOFT_DELIVERY_OPTIMIZATION: HeaderValue =
            HeaderValue::from_static("Microsoft-Delivery-Optimization/10.1");
        const SEC_GPC: &str = "Sec-GPC";

        let mut headers = HeaderMap::with_capacity(3);
        headers.insert(USER_AGENT, MICROSOFT_DELIVERY_OPTIMIZATION);
        headers.insert(DNT, HeaderValue::from(1));
        headers.insert(SEC_GPC, HeaderValue::from(1));
        headers
    }

    pub async fn fetch(
        &self,
        client: &Client,
        mut download: Download,
        multi_progress: &MultiProgress,
    ) -> Result<DownloadedFile> {
        download.convert_to_github_versioned().await?;

        download.upgrade_to_https(client).await;

        let res = client.get((***download.url()).clone()).send().await?;

        if let Err(err) = res.error_for_status_ref() {
            bail!(
                "{} returned {}",
                err.url().unwrap().as_str(),
                err.status().unwrap()
            )
        }

        // Check that we're downloading an application
        if let Some(content_type) = res.headers().get(CONTENT_TYPE)
            && !content_type
                .as_bytes()
                .starts_with(Self::APPLICATION.as_bytes())
        {
            bail!(
                "The content type for {download} was {content_type:?} but an {application} content type was expected",
                application = Self::APPLICATION
            );
        }

        let file_name = download
            .file_name(res.url(), res.headers().get(CONTENT_DISPOSITION))
            .into_owned();

        let last_modified = res
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|last_modified| last_modified.to_str().ok())
            .and_then(|last_modified| DateTime::parse_from_rfc2822(last_modified).ok())
            .map(|date_time| date_time.date_naive());

        let progress_bar = match res.content_length() {
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

        let mut stream = res.bytes_stream();

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
            url: download.into_url(),
            mmap: unsafe { Mmap::map(&temp_file) }?,
            file: temp_file,
            sha_256: Sha256String::from_digest(&sha_256),
            file_name,
            last_modified,
        })
    }
}
