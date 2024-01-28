use crate::msix_family::msix::APPX_SIGNATURE_P7X;
use color_eyre::eyre::Result;
use sha2::{Digest, Sha256};
use std::io::{Read, Seek};
use zip::ZipArchive;

pub fn read_manifest<R: Read + Seek>(zip: &mut ZipArchive<R>, path: &str) -> Result<String> {
    let mut appx_manifest_file = zip.by_name(path)?;
    let mut appx_manifest = String::with_capacity(usize::try_from(appx_manifest_file.size())?);
    appx_manifest_file.read_to_string(&mut appx_manifest)?;
    Ok(appx_manifest)
}

pub fn hash_signature<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<String> {
    let mut signature_file = zip.by_name(APPX_SIGNATURE_P7X)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1 << 13];
    loop {
        let count = signature_file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(base16ct::upper::encode_string(&hasher.finalize()))
}
