use crate::zip::get_entry_file_name;
use async_zip::tokio::read::seek::ZipFileReader;
use color_eyre::eyre::{bail, Result};
use futures_util::AsyncReadExt;
use tokio::fs::File;

const APPX_SIGNATURE_P7X: &str = "AppxSignature.p7x";

pub async fn get_manifest_and_signature(
    mut zip: ZipFileReader<&mut File>,
    manifest_path: &str,
) -> Result<(String, Vec<u8>)> {
    let mut appx_manifest = String::new();
    let mut appx_signature = Vec::new();

    for index in 0..zip.file().entries().len() {
        match get_entry_file_name(&zip, index) {
            file_name if file_name == manifest_path => {
                zip.reader_without_entry(index)
                    .await?
                    .read_to_string(&mut appx_manifest)
                    .await?;
            }
            APPX_SIGNATURE_P7X => {
                zip.reader_without_entry(index)
                    .await?
                    .read_to_end(&mut appx_signature)
                    .await?;
            }
            _ => {}
        };
        if !appx_manifest.is_empty() && !appx_signature.is_empty() {
            break;
        }
    }

    if appx_manifest.is_empty() {
        bail!("No {manifest_path} was found in the Msix file")
    }

    if appx_signature.is_empty() {
        bail!("No {APPX_SIGNATURE_P7X} was found in the Msix file")
    }

    Ok((appx_manifest, appx_signature))
}
