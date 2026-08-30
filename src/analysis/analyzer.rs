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

use super::extensions::FileExtension;
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
        let installers = match Utf8Path::new(file_name)
            .extension()
            .unwrap_or_default()
            .parse()?
        {
            FileExtension::Msi => Msi::new(reader)?.installers(),
            FileExtension::Msix | FileExtension::Appx => Msix::new(reader)?.installers(),
            FileExtension::MsixBundle | FileExtension::AppxBundle => {
                MsixBundle::new(reader)?.installers()
            }
            FileExtension::Zip => {
                let mut scoped_zip = Zip::new(reader)?;
                let installers = mem::take(&mut scoped_zip.installers);
                return Ok(Self {
                    installers,
                    zip: Some(scoped_zip),
                    ..Self::default()
                });
            }
            FileExtension::Exe => {
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
            FileExtension::AppInstaller => {
                // AppInstaller files will only reach this point from the analyze command as they
                // are converted to an MSIX or MSIXBundle before downloading
                bail!(".appinstaller files are not supported for the analyze command")
            }
        };
        Ok(Self {
            installers,
            ..Self::default()
        })
    }

    /// Consumes the [`Analyzer`], returning the inner installers.
    pub fn into_installers(self) -> Vec<Installer> {
        self.installers
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
