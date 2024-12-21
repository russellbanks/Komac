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
    // 将 GitHub 最新版本 URL 转换为版本化的 URL
    convert_github_latest_to_versioned(&mut url).await?;

    // 如果可访问，则升级到 HTTPS
    upgrade_to_https_if_reachable(&mut url, client).await?;

    let res = client.get(url.as_str()).send().await?;

    if let Err(err) = res.error_for_status_ref() {
        bail!(
            "{} 返回 {}",
            err.url().unwrap().as_str(),
            err.status().unwrap()
        )
    }

    let content_disposition = res.headers().get(CONTENT_DISPOSITION);
    let file_name = get_file_name(&url, res.url(), content_disposition);
    let total_size = res
        .content_length()
        .ok_or_else(|| eyre!("无法从 '{url}' 获取内容长度"))?;

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
        .with_message(format!("正在下载 {url}"))
    );

    // 下载块
    let temp_file = tempfile::tempfile()?;
    let mut file = tokio::fs::File::from_std(temp_file.try_clone()?);
    let mut downloaded = 0;
    let mut stream = res.bytes_stream();

    let mut hasher = Sha256::new();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        let write = file.write_all(&chunk);
        hasher.update(&chunk); // 在下载时对文件进行哈希
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

/// 从 URL 获取文件名，给定 URL、最终重定向的 URL 和可选的 Content-Disposition 头。
///
/// 这通过从 Content-Disposition 头获取文件名来实现。它旨在模仿 Firefox 的功能，即使同时提供了 filename 和 filename* 参数，也优先考虑 filename* 参数。
/// 参见 [Content-Disposition](https://developer.mozilla.org/docs/Web/HTTP/Headers/Content-Disposition)。
///
/// 如果没有 Content-Disposition 头或 Content-Disposition 中没有文件名，则回退到获取初始 URL 的最后一部分，然后是最终重定向的 URL（如果初始 URL 末尾没有有效的文件扩展名）。
fn get_file_name(url: &Url, final_url: &Url, content_disposition: Option<&HeaderValue>) -> String {
    const FILENAME: &str = "filename";
    const FILENAME_EXT: &str = formatcp!("{FILENAME}*");

    if let Some(content_disposition) = content_disposition.and_then(|value| value.to_str().ok()) {
        let mut sections = content_disposition.split(';');
        let _disposition = sections.next(); // 跳过处置类型
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

    // 如果没有 Content-Disposition 头或 Content-Disposition 中没有文件名，则回退
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

/// 通过一次跳转跟踪，将始终指向最新版本的 GitHub 虚荣 URL 转换为版本化的 URL。
///
/// 例如，github.com/owner/repo/releases/latest/download/file.exe 转换为
/// github.com/owner/repo/releases/download/v1.2.3/file.exe
async fn convert_github_latest_to_versioned(url: &mut Url) -> Result<()> {
    const LATEST: &str = "latest";
    const DOWNLOAD: &str = "download";
    const MAX_HOPS: u8 = 2;

    if url.host_str() != Some(GITHUB_HOST) {
        return Ok(());
    }

    if let Some(mut segments) = url.path_segments() {
        // 如果第 4 和第 5 段是 'latest' 和 'download'，则它是虚荣 URL
        if segments.nth(3) == Some(LATEST) && segments.next() == Some(DOWNLOAD) {
            // 创建一个只会重定向一次的客户端
            let limited_redirect_client = ClientBuilder::new()
                .redirect(Policy::limited(MAX_HOPS as usize))
                .build()?;

            // 如果由于达到最大跳数而出现重定向错误，则将原始虚荣 URL 设置为重定向的版本化 URL
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
    // 由于下载的文件是临时文件，因此将其存储在此处，以便引用保持活动状态，文件不会被删除。
    // 这是必要的，因为内存映射需要对文件的引用。
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
