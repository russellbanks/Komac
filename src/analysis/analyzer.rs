use std::{
    io::{Read, Seek},
    mem,
};

use camino::Utf8Path;
use color_eyre::eyre::{Result, bail};
use winget_types::{
    installer::Installer,
    locale::{Copyright, PackageName, Publisher},
};

use super::extensions::{APPX, APPX_BUNDLE, EXE, MSI, MSIX, MSIX_BUNDLE, ZIP};
use crate::analysis::{
    Installers,
    installers::{
        Exe, Msi, Zip,
        msix_family::{Msix, bundle::MsixBundle},
    },
};

pub struct Analyzer<'reader, R: Read + Seek> {
    pub file_name: String,
    pub copyright: Option<Copyright>,
    pub package_name: Option<PackageName>,
    pub publisher: Option<Publisher>,
    pub installers: Vec<Installer>,
    pub zip: Option<Zip<&'reader mut R>>,
}

impl<'reader, R: Read + Seek> Analyzer<'reader, R> {
    pub fn new(reader: &'reader mut R, file_name: &str) -> Result<Self> {
        let extension = Utf8Path::new(file_name)
            .extension()
            .unwrap_or_default()
            .to_ascii_lowercase();

        let installers = match extension.as_str() {
            MSI => Msi::new(reader)?.installers(),
            MSIX | APPX => Msix::new(reader)?.installers(),
            MSIX_BUNDLE | APPX_BUNDLE => MsixBundle::new(reader)?.installers(),
            ZIP => {
                let mut scoped_zip = Zip::new(reader)?;
                let installers = mem::take(&mut scoped_zip.installers);
                return Ok(Self {
                    installers,
                    zip: Some(scoped_zip),
                    ..Self::default()
                });
            }
            EXE => {
                let mut exe = Exe::new(reader)?;
                return Ok(Self {
                    installers: exe.installers(),
                    copyright: exe
                        .legal_copyright
                        .take()
                        .and_then(|copyright| Copyright::new(copyright).ok()),
                    package_name: exe
                        .product_name
                        .take()
                        .and_then(|product_name| PackageName::new(product_name).ok()),
                    publisher: exe
                        .company_name
                        .take()
                        .and_then(|company_name| Publisher::new(company_name).ok()),
                    ..Self::default()
                });
            }
            _ => bail!(r#"Unsupported file extension: "{extension}""#),
        };
        Ok(Self {
            installers,
            ..Self::default()
        })
    }
}

impl<R: Read + Seek> Default for Analyzer<'_, R> {
    fn default() -> Self {
        Self {
            file_name: String::default(),
            copyright: None,
            package_name: None,
            publisher: None,
            installers: Vec::default(),
            zip: None,
        }
    }
}
