use std::{cmp::min, collections::HashMap, fs::File, mem, num::NonZeroU8};

use camino::Utf8Path;
use chrono::{DateTime, NaiveDate};
use color_eyre::eyre::{Result, bail, eyre};
use const_format::formatcp;
use futures_util::{StreamExt, TryStreamExt, stream};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools;
use memmap2::Mmap;
use reqwest::{
    Client,
    header::{CONTENT_DISPOSITION, HeaderValue, LAST_MODIFIED},
};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use url::Url;
use uuid::Uuid;
use winget_types::{
    installer::{Architecture, VALID_FILE_EXTENSIONS},
    shared::{Sha256String, url::DecodedUrl},
};

use crate::{
    file_analyser::FileAnalyser,
    traits::url::{ConvertGitHubLatestToVersioned, UpgradeToHttps},
};

async fn download_file(
    client: &Client,
    mut url: DecodedUrl,
    multi_progress: &MultiProgress,
) -> Result<DownloadedFile> {
    url.convert_github_latest_to_versioned().await?;

    url.upgrade_to_https(client).await;

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
            .template("{msg}\n{wide_bar:.magenta/black} {decimal_bytes:.green}/{decimal_total_bytes:.green} {decimal_bytes_per_sec:.red} eta {eta:.blue}")?
            .progress_chars("───")
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

pub async fn download_urls(
    client: &Client,
    urls: Vec<DecodedUrl>,
    concurrent_downloads: NonZeroU8,
) -> Result<Vec<DownloadedFile>> {
    let multi_progress = MultiProgress::new();
    let downloaded_files = stream::iter(
        urls.into_iter()
            .unique()
            .map(|url| download_file(client, url, &multi_progress)),
    )
    .buffer_unordered(concurrent_downloads.get() as usize)
    .try_collect::<Vec<_>>()
    .await?;
    multi_progress.clear()?;
    Ok(downloaded_files)
}

pub struct DownloadedFile {
    // As the downloaded file is a temporary file, it's stored here so that the reference stays
    // alive and the file does not get deleted. This is necessary because the memory map needs the
    // file to remain present.
    #[expect(dead_code)]
    file: File,
    url: DecodedUrl,
    mmap: Mmap,
    sha_256: Sha256String,
    file_name: String,
    last_modified: Option<NaiveDate>,
}

pub async fn process_files(
    files: &mut [DownloadedFile],
) -> Result<HashMap<DecodedUrl, FileAnalyser>> {
    stream::iter(files.iter_mut().map(
        |DownloadedFile {
             url,
             mmap,
             sha_256,
             file_name,
             last_modified,
             ..
         }| async move {
            let mut file_analyser = FileAnalyser::new(mmap, file_name)?;
            let architecture_in_url = Architecture::from_url(url.as_str());
            for installer in &mut file_analyser.installers {
                if let Some(architecture) = architecture_in_url {
                    installer.architecture = architecture;
                }
                installer.url = url.clone();
                installer.sha_256 = sha_256.clone();
                installer.release_date = *last_modified;
            }
            file_analyser.file_name = mem::take(file_name);
            Ok((mem::take(url), file_analyser))
        },
    ))
    .buffer_unordered(num_cpus::get())
    .try_collect::<HashMap<_, _>>()
    .await
}
