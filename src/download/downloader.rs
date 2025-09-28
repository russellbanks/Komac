use std::num::NonZeroUsize;

use chrono::DateTime;
use color_eyre::{
    Result,
    eyre::{bail, eyre},
};
use futures_util::{StreamExt, TryStreamExt, stream};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use memmap2::Mmap;
use reqwest::{
    Client,
    header::{CONTENT_DISPOSITION, DNT, HeaderMap, HeaderValue, LAST_MODIFIED, USER_AGENT},
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
    concurrent_downloads: NonZeroUsize,
}

impl Downloader {
    const PROGRESS_TEMPLATE: &'static str = "{msg}\n{wide_bar:.magenta/black} {decimal_bytes:.green}/{decimal_total_bytes:.green} {decimal_bytes_per_sec:.red} eta {eta:.blue}";

    const PROGRESS_CHARS: &'static str = "───";

    pub const fn new_with_concurrent(concurrent_downloads: NonZeroUsize) -> Self {
        Self {
            concurrent_downloads,
        }
    }

    pub async fn download(&self, downloads: &[Download]) -> Result<Vec<DownloadedFile>> {
        let client = Client::builder().default_headers(Self::headers()).build()?;

        let multi_progress = MultiProgress::new();

        let downloaded_files = stream::iter(downloads)
            .map(|download| self.fetch(&client, download.clone(), &multi_progress))
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

        let mut headers = HeaderMap::new();
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

        let res = client.get(download.url.as_str()).send().await?;

        if let Err(err) = res.error_for_status_ref() {
            bail!(
                "{} returned {}",
                err.url().unwrap().as_str(),
                err.status().unwrap()
            )
        }

        let content_disposition = res.headers().get(CONTENT_DISPOSITION);
        let file_name = download.file_name(res.url(), content_disposition);
        let total_size = res
            .content_length()
            .ok_or_else(|| eyre!("Failed to get content length from '{}'", download.url))?;

        let last_modified = res
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|last_modified| last_modified.to_str().ok())
            .and_then(|last_modified| DateTime::parse_from_rfc2822(last_modified).ok())
            .map(|date_time| date_time.date_naive());

        let progress = multi_progress.add(
            ProgressBar::new(total_size)
                .with_style(
                    ProgressStyle::default_bar()
                        .template(Self::PROGRESS_TEMPLATE)?
                        .progress_chars(Self::PROGRESS_CHARS),
                )
                .with_message(format!("Downloading {}", download.url)),
        );

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
            (Ok(_), sha_256) => sha_256,
            (Err(err), _) => return Err(err.into()),
        };

        progress.finish();

        Ok(DownloadedFile {
            url: download.url,
            mmap: unsafe { Mmap::map(&temp_file) }?,
            file: temp_file,
            sha_256: Sha256String::from_digest(&sha_256),
            file_name,
            last_modified,
        })
    }
}
