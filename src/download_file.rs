use camino::Utf8Path;
use chrono::{DateTime, NaiveDate};
use color_eyre::eyre::{bail, eyre, Result};
use const_format::formatcp;
use futures_util::{stream, StreamExt, TryStreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools;
use memmap2::Mmap;
use reqwest::header::{HeaderValue, CONTENT_DISPOSITION, LAST_MODIFIED};
use reqwest::redirect::Policy;
use reqwest::{Client, ClientBuilder, Response};
use sha2::{Digest, Sha256};
use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::future::Future;
use std::mem;
use tokio::io::AsyncWriteExt;
use url::Url;
use uuid::Uuid;

use crate::file_analyser::FileAnalyser;
use crate::github::github_client::GITHUB_HOST;
use crate::types::architecture::{Architecture, VALID_FILE_EXTENSIONS};
use crate::types::sha_256::Sha256String;
use crate::types::urls::url::DecodedUrl;

async fn download_file(
    client: &Client,
    mut url: DecodedUrl,
    multi_progress: &MultiProgress,
) -> Result<DownloadedFile> {
    convert_github_latest_to_versioned(&mut url).await?;

    upgrade_to_https_if_reachable(&mut url, client).await?;

    let res = client.get(url.as_str()).send().await?;

    if let Err(err) = res.error_for_status_ref() {
        bail!(
            "{} returned {}",
            err.url().unwrap().as_str(),
            err.status().unwrap()
        )
    }

    let content_disposition = res.headers().get(CONTENT_DISPOSITION);
    let file_name = get_file_name(&url, res.url(), content_disposition);
    let total_size = res
        .content_length()
        .ok_or_else(|| eyre!("Failed to get content length from '{url}'"))?;

    let last_modified = res
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|last_modified| last_modified.to_str().ok())
        .and_then(|last_modified| DateTime::parse_from_rfc2822(last_modified).ok())
        .map(|date_time| date_time.date_naive());

    let pb = multi_progress.add(ProgressBar::new(total_size)
        .with_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-")
        )
        .with_message(format!("Downloading {url}"))
    );

    // Download chunks
    let temp_file = tempfile::tempfile()?;
    let mut file = tokio::fs::File::from_std(temp_file.try_clone()?);
    let mut downloaded = 0;
    let mut stream = res.bytes_stream();

    let mut hasher = Sha256::new();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        let write = file.write_all(&chunk);
        hasher.update(&chunk); // Hash file as it's downloading
        downloaded = min(downloaded + (chunk.len() as u64), total_size);
        pb.set_position(downloaded);
        write.await?;
    }
    file.flush().await?;
    file.sync_all().await?;
    pb.finish_and_clear();

    Ok(DownloadedFile {
        url,
        mmap: unsafe { Mmap::map(&temp_file) }?,
        file: temp_file,
        sha_256: Sha256String::from_hasher(&hasher.finalize())?,
        file_name,
        last_modified,
    })
}

/// Gets the filename from a URL given the URL, a final redirected URL, and an optional
/// Content-Disposition header.
///
/// This works by getting the filename from the Content-Disposition header. It aims to mimic
/// Firefox's functionality whereby the filename* parameter is prioritised over filename even if
/// both are provided. See [Content-Disposition](https://developer.mozilla.org/docs/Web/HTTP/Headers/Content-Disposition).
///
/// If there is no Content-Disposition header or no filenames in the Content-Disposition, it falls
/// back to getting the last part of the initial URL and then the final redirected URL if the
/// initial URL does not have a valid file extension at the end.
fn get_file_name(url: &Url, final_url: &Url, content_disposition: Option<&HeaderValue>) -> String {
    const FILENAME: &str = "filename";
    const FILENAME_EXT: &str = formatcp!("{FILENAME}*");

    if let Some(content_disposition) = content_disposition.and_then(|value| value.to_str().ok()) {
        let mut sections = content_disposition.split(';');
        let _disposition = sections.next(); // Skip the disposition type
        let filenames = sections
            .filter_map(|section| {
                if let Some((key, value)) = section
                    .split_once('=')
                    .map(|(key, value)| (key.trim(), value.trim()))
                {
                    if key.starts_with(FILENAME) {
                        let value = value.trim_matches('"').trim();
                        if !value.is_empty() {
                            return Some((key, value));
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>();

        let filename = filenames
            .iter()
            .find_map(|&(key, value)| (key == FILENAME_EXT).then_some(value))
            .or_else(|| {
                filenames
                    .into_iter()
                    .find_map(|(key, value)| (key == FILENAME).then_some(value))
            });
        if let Some(filename) = filename {
            return filename.to_owned();
        }
    }

    // Fallback if there is no Content-Disposition header or no filenames in Content-Disposition
    url.path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|last_segment| {
            Utf8Path::new(last_segment)
                .extension()
                .is_some_and(|extension| VALID_FILE_EXTENSIONS.contains(&extension))
        })
        .or_else(|| {
            final_url
                .path_segments()
                .and_then(|mut segments| segments.next_back())
        })
        .map_or_else(|| Uuid::new_v4().to_string(), str::to_owned)
}

async fn upgrade_to_https_if_reachable(url: &mut Url, client: &Client) -> Result<()> {
    if url.scheme() == "http" {
        url.set_scheme("https").unwrap();
        if client
            .head(url.as_str())
            .send()
            .await
            .and_then(Response::error_for_status)
            .is_err()
        {
            url.set_scheme("http").unwrap();
        }
    }
    Ok(())
}

/// Converts a vanity GitHub URL that always points to the latest release to its versioned URL by
/// following the redirect by one hop.
///
/// For example, github.com/owner/repo/releases/latest/download/file.exe to
/// github.com/owner/repo/releases/download/v1.2.3/file.exe
async fn convert_github_latest_to_versioned(url: &mut Url) -> Result<()> {
    const LATEST: &str = "latest";
    const DOWNLOAD: &str = "download";
    const MAX_HOPS: u8 = 2;

    if url.host_str() != Some(GITHUB_HOST) {
        return Ok(());
    }

    if let Some(mut segments) = url.path_segments() {
        // If the 4th and 5th segments are 'latest' and 'download', it's a vanity URL
        if segments.nth(3) == Some(LATEST) && segments.next() == Some(DOWNLOAD) {
            // Create a client that will redirect only once
            let limited_redirect_client = ClientBuilder::new()
                .redirect(Policy::limited(MAX_HOPS as usize))
                .build()?;

            // If there was a redirect error because max hops were reached, as intended, set the
            // original vanity URL to the redirected versioned URL
            if let Err(error) = limited_redirect_client.head(url.as_str()).send().await {
                if error.is_redirect() {
                    if let Some(final_url) = error.url() {
                        *url = final_url.clone();
                    }
                }
            };
        }
    }
    Ok(())
}

pub fn download_urls<'a>(
    client: &'a Client,
    urls: Vec<DecodedUrl>,
    multi_progress: &'a MultiProgress,
) -> impl Iterator<Item = impl Future<Output = Result<DownloadedFile>> + 'a> {
    urls.into_iter()
        .unique()
        .map(|url| download_file(client, url, multi_progress))
}

pub struct DownloadedFile {
    pub url: DecodedUrl,
    pub mmap: Mmap,
    // As the downloaded file is a temporary file, it's stored here so that the reference stays
    // alive and the file does not get deleted. This is necessary because the memory map needs the
    // reference to the file.
    #[expect(dead_code)]
    pub file: File,
    pub sha_256: Sha256String,
    pub file_name: String,
    pub last_modified: Option<NaiveDate>,
}

pub async fn process_files(
    files: &mut [DownloadedFile],
) -> Result<HashMap<DecodedUrl, FileAnalyser>> {
    stream::iter(files.iter_mut().map(
        |DownloadedFile {
             url,
             file: _,
             mmap,
             sha_256,
             file_name,
             last_modified,
         }| async move {
            let mut file_analyser = FileAnalyser::new(mmap, file_name)?;
            if let Some(architecture) = Architecture::get_from_url(url.as_str()) {
                file_analyser.installer.architecture = architecture;
            }
            file_analyser.installer.installer_sha_256 = mem::take(sha_256);
            file_analyser.installer.release_date = last_modified.take();
            file_analyser.file_name = mem::take(file_name);
            Ok((mem::take(url), file_analyser))
        },
    ))
    .buffered(num_cpus::get())
    .try_collect::<HashMap<_, _>>()
    .await
}
