mod manifest;

use std::{
    borrow::Cow,
    io,
    io::{BufReader, Read, Seek},
};

use color_eyre::Result;
use manifest::{Bundle, Identity, Package};
use percent_encoding::percent_decode_str;
use quick_xml::{Reader, events::Event};
use winget_types::{
    Sha256String,
    installer::{Installer, PackageFamilyName},
};
use zip::ZipArchive;

use super::{
    Msix,
    utils::{hash_signature, read_manifest},
};
use crate::analysis::Installers;

pub struct MsixBundle {
    pub signature_sha_256: Sha256String,
    pub package_family_name: PackageFamilyName<'static>,
    pub msix_files: Vec<Msix>,
}

const APPX_BUNDLE_MANIFEST_PATH: &str = "AppxMetadata/AppxBundleManifest.xml";

impl MsixBundle {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let appx_bundle_manifest = read_manifest(&mut zip, APPX_BUNDLE_MANIFEST_PATH)?;

        let signature_sha_256 = hash_signature(&mut zip)?;

        let mut bundle = Bundle::default();

        let mut reader = Reader::from_str(&appx_bundle_manifest);
        let config = reader.config_mut();
        config.expand_empty_elements = true;
        config.trim_text(true);

        loop {
            match reader.read_event()? {
                Event::Start(event) => match event.local_name().as_ref() {
                    b"Identity" => bundle.identity = Identity::from_event(event, &mut reader)?,
                    b"Package" => bundle
                        .packages
                        .push(Package::from_event(event, &mut reader)?),
                    _ => {}
                },
                Event::Eof => break,
                _ => {}
            }
        }

        let package_family_name = PackageFamilyName::new(
            bundle.identity.name().to_owned(),
            bundle.identity.publisher(),
        );

        Ok(Self {
            msix_files: bundle
                .packages
                .iter()
                .filter(|package| package.is_application() && !package.is_stub())
                .map(|package| {
                    // Find file by package file name, comparing by decoded file names
                    let file_name = zip
                        .file_names()
                        .find(|file_name| {
                            percent_decode_str(file_name)
                                .eq(percent_decode_str(package.file_name()))
                        })
                        .map(|file_name| Cow::Owned(file_name.to_owned()))
                        .unwrap_or(Cow::Borrowed(package.file_name()));

                    let mut embedded_msix = zip.by_name(&file_name)?;
                    let mut temp_file = tempfile::tempfile()?;
                    io::copy(&mut embedded_msix, &mut temp_file)?;
                    Msix::new(BufReader::new(temp_file))
                })
                .collect::<Result<Vec<_>>>()?,
            signature_sha_256,
            package_family_name,
        })
    }
}

impl Installers for MsixBundle {
    fn installers(&self) -> Vec<Installer> {
        self.msix_files
            .iter()
            .map(|msix| Installer {
                signature_sha_256: Some(self.signature_sha_256.clone()),
                package_family_name: Some(self.package_family_name.clone()),
                ..msix.installers().swap_remove(0)
            })
            .collect::<Vec<_>>()
    }
}
