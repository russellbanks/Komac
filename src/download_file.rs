use crate::file_analyser::{get_architecture, FileAnalyser};
use crate::manifests::installer_manifest::{Architecture, InstallerType};
use crate::msi::Msi;
use crate::msix_family::msix::Msix;
use crate::msix_family::msixbundle::MsixBundle;
use crate::types::urls::url::Url;
use crate::url_utils::find_architecture;
use async_tempfile::TempFile;
use color_eyre::eyre::{eyre, Result, WrapErr};
use futures_util::{stream, StreamExt, TryStreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools;
use reqwest::header::{HeaderValue, CONTENT_DISPOSITION, LAST_MODIFIED};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::cmp::min;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::future::Future;
use time::format_description::well_known::Rfc2822;
use time::{Date, OffsetDateTime};
use tokio::io::AsyncWriteExt;
use xxhash_rust::xxh3::xxh3_64;

async fn download_file<'url>(
    client: &Client,
    url: &'url str,
    multi_progress: &MultiProgress,
) -> Result<DownloadedFile> {
    let res = client
        .get(url)
        .send()
        .await
        .wrap_err_with(|| format!("Failed to GET from '{url}'"))?;

    let content_disposition = res.headers().get(CONTENT_DISPOSITION);
    let filename = get_file_name(url, content_disposition);
    let total_size = res
        .content_length()
        .ok_or_else(|| eyre!("Failed to get content length from '{url}'"))?;

    let last_modified = res
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|last_modified| last_modified.to_str().ok())
        .and_then(|last_modified| OffsetDateTime::parse(last_modified, &Rfc2822).ok())
        .map(OffsetDateTime::date);

    let pb = multi_progress.add(ProgressBar::new(total_size)
        .with_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-")
        )
        .with_message(format!("Downloading {url}"))
    );

    // Download chunks
    let temp_file = TempFile::new_with_name(filename).await?;
    let mut file = temp_file.open_rw().await?;
    let mut downloaded = 0;
    let mut stream = res.bytes_stream();

    let mut hasher = Sha256::new();
    while let Some(item) = stream.next().await {
        let chunk = item.wrap_err_with(|| "Error while downloading file")?;
        let write = file.write_all(&chunk);
        hasher.update(&chunk); // Hash file as it's downloading
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
        write.await?;
    }
    pb.finish_and_clear();

    Ok(DownloadedFile {
        url: Url::parse(url)?,
        file: temp_file.open_ro().await?,
        sha_256: base16ct::upper::encode_string(&hasher.finalize()),
        last_modified,
    })
}

fn get_file_name(url: &str, content_disposition: Option<&HeaderValue>) -> String {
    if let Some(content_disposition) = content_disposition.map(|s| s.to_str().unwrap_or_default()) {
        let mut sections = content_disposition.split(';');
        let _disposition = sections.next();
        for section in sections {
            let mut parts = section.splitn(2, '=');

            let key = parts.next().map(str::trim);
            let value = parts.next().map(str::trim);
            if let (Some(key), Some(value)) = (key, value) {
                if key.starts_with("filename") {
                    return value.trim_matches('"').to_owned();
                }
            } else {
                break;
            }
        }
    }
    if let Some((_, file_name)) = url.rsplit_once('/') {
        file_name.to_owned()
    } else {
        xxh3_64(url.as_bytes()).to_string()
    }
}

pub fn download_urls<'a>(
    client: &'a Client,
    urls: &'a [Url],
    multi_progress: &'a MultiProgress,
) -> impl Iterator<Item = impl Future<Output = Result<DownloadedFile>> + 'a> {
    urls.iter()
        .unique()
        .map(|url| download_file(client, url.as_str(), multi_progress))
}

pub struct DownloadedFile {
    pub url: Url,
    pub file: TempFile,
    pub sha_256: String,
    pub last_modified: Option<Date>,
}

pub struct InitialData {
    pub architecture: Architecture,
    pub installer_type: InstallerType,
    pub installer_sha_256: String,
    pub last_modified: Option<Date>,
    pub msi: Option<Msi>,
    pub msix: Option<Msix>,
    pub msix_bundle: Option<MsixBundle>,
    pub file_name: String,
}

pub async fn process_files(files: Vec<DownloadedFile>) -> Result<HashMap<Url, InitialData>> {
    stream::iter(files.into_iter().map(
        |DownloadedFile {
             url,
             mut file,
             sha_256,
             last_modified,
         }| async move {
            let file_analyser = FileAnalyser::new(&mut file).await?;
            let initial_data = InitialData {
                architecture: find_architecture(url.as_str())
                    .or_else(|| file_analyser.msi.as_ref().map(|msi| msi.architecture))
                    .or_else(|| {
                        file_analyser
                            .msix
                            .as_ref()
                            .map(|msix| msix.processor_architecture)
                    })
                    .unwrap_or_else(|| get_architecture(&file_analyser.pe.unwrap()).unwrap()),
                installer_type: file_analyser.installer_type,
                installer_sha_256: sha_256,
                last_modified,
                msi: file_analyser.msi,
                msix: file_analyser.msix,
                msix_bundle: file_analyser.msix_bundle,
                file_name: file
                    .file_path()
                    .file_name()
                    .and_then(OsStr::to_str)
                    .map(str::to_owned)
                    .unwrap(),
            };
            Ok((url, initial_data))
        },
    ))
    .buffered(num_cpus::get())
    .try_collect::<HashMap<_, _>>()
    .await
}
