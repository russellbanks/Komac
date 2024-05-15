use std::borrow::Cow;
use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::future::Future;

use camino::Utf8Path;
use chrono::{DateTime, NaiveDate};
use color_eyre::eyre::{bail, eyre, Result};
use const_format::formatcp;
use futures_util::{stream, StreamExt, TryStreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools;
use memmap2::Mmap;
use reqwest::header::{HeaderValue, CONTENT_DISPOSITION, LAST_MODIFIED};
use reqwest::{Client, Response};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::file_analyser::FileAnalyser;
use crate::types::architecture::{Architecture, VALID_FILE_EXTENSIONS};
use crate::types::urls::url::Url;

async fn download_file(
    client: &Client,
    mut url: url::Url,
    multi_progress: &MultiProgress,
) -> Result<DownloadedFile> {
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
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
        write.await?;
    }
    pb.finish_and_clear();

    Ok(DownloadedFile {
        url: url.into(),
        file: temp_file,
        sha_256: base16ct::upper::encode_string(&hasher.finalize()),
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
fn get_file_name(
    url: &url::Url,
    final_url: &url::Url,
    content_disposition: Option<&HeaderValue>,
) -> String {
    const FILENAME: &str = "filename";
    const FILENAME_EXT: &str = formatcp!("{FILENAME}*");

    if let Some(content_disposition) = content_disposition.and_then(|value| value.to_str().ok()) {
        let mut sections = content_disposition.split(';');
        let _disposition = sections.next(); // Skip the disposition type
        let filenames = sections
            .filter_map(|section| {
                let mut parts = section.splitn(2, '=').map(str::trim);
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
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
                .map_or(false, |extension| {
                    VALID_FILE_EXTENSIONS.contains(&extension)
                })
        })
        .or_else(|| {
            final_url
                .path_segments()
                .and_then(|mut segments| segments.next_back())
        })
        .map_or_else(|| Uuid::new_v4().to_string(), str::to_owned)
}

pub fn download_urls<'a>(
    client: &'a Client,
    urls: Vec<Url>,
    multi_progress: &'a MultiProgress,
) -> impl Iterator<Item = impl Future<Output = Result<DownloadedFile>> + 'a> {
    urls.into_iter()
        .unique()
        .map(|url| download_file(client, url.into_inner(), multi_progress))
}

pub struct DownloadedFile {
    pub url: Url,
    pub file: File,
    pub sha_256: String,
    pub file_name: String,
    pub last_modified: Option<NaiveDate>,
}

pub async fn process_files<'a>(
    files: Vec<DownloadedFile>,
) -> Result<HashMap<Url, FileAnalyser<'a>>> {
    stream::iter(files.into_iter().map(
        |DownloadedFile {
             url,
             file,
             sha_256,
             file_name,
             last_modified,
         }| async move {
            let map = unsafe { Mmap::map(&file) }?;
            let mut file_analyser = FileAnalyser::new(map.as_ref(), Cow::Owned(file_name))?;
            file_analyser.architecture =
                Architecture::get_from_url(url.as_str()).or(file_analyser.architecture);
            file_analyser.installer_sha_256 = sha_256;
            file_analyser.last_modified = last_modified;
            Ok((url, file_analyser))
        },
    ))
    .buffered(num_cpus::get())
    .try_collect::<HashMap<_, _>>()
    .await
}
