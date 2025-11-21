use std::{io::Cursor, mem};

use camino::Utf8Path;
use color_eyre::eyre::{Result, bail};
use memmap2::Mmap;
use winget_types::{
    installer::Installer,
    locale::{Copyright, PackageName, Publisher},
};

use super::extensions::{APPX, APPX_BUNDLE, MSI, MSIX, MSIX_BUNDLE, ZIP};
use crate::analysis::{
    Installers,
    installers::{
        Exe, Msi, Zip,
        msix_family::{Msix, bundle::MsixBundle},
    },
};

pub struct Analyzer<'data> {
    pub file_name: String,
    pub copyright: Option<Copyright>,
    pub package_name: Option<PackageName>,
    pub publisher: Option<Publisher>,
    pub installers: Vec<Installer>,
    pub zip: Option<Zip<Cursor<&'data [u8]>>>,
}

impl<'data> Analyzer<'data> {
    pub fn new(data: &'data Mmap, file_name: &str) -> Result<Self> {
        let extension = Utf8Path::new(file_name).extension().unwrap_or_default();

        let mut zip = None;
        let mut copyright = None;
        let mut package_name = None;
        let mut publisher = None;
        let installers = match extension {
            MSI => Msi::new(Cursor::new(data.as_ref()))?.installers(),
            MSIX | APPX => Msix::new(Cursor::new(data.as_ref()))?.installers(),
            MSIX_BUNDLE | APPX_BUNDLE => MsixBundle::new(Cursor::new(data.as_ref()))?.installers(),
            ZIP => {
                let mut scoped_zip = Zip::new(Cursor::new(data.as_ref()))?;
                let installers = mem::take(&mut scoped_zip.installers);
                zip = Some(scoped_zip);
                installers
            }
            EXE => Exe::new(Cursor::new(data.as_ref()))?.installers(),
            _ => bail!(r#"Unsupported file extension: "{extension}""#),
        };
        Ok(Self {
            installers,
            file_name: String::new(),
            copyright,
            package_name,
            publisher,
            zip,
        })
    }
}
