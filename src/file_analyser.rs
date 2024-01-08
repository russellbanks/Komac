use crate::manifests::installer_manifest::Platform;
use crate::msi::Msi;
use crate::msix_family::msix::Msix;
use crate::msix_family::msixbundle::MsixBundle;
use crate::types::architecture::Architecture;
use crate::types::copyright::Copyright;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use crate::types::minimum_os_version::MinimumOSVersion;
use crate::types::package_name::PackageName;
use crate::types::publisher::Publisher;
use crate::zip::Zip;
use async_recursion::async_recursion;
use async_tempfile::TempFile;
use color_eyre::eyre::Result;
use exe::ResolvedDirectoryID::Name;
use exe::{PETranslation, ResourceDirectory, VSVersionInfo, VecPE, PE};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::SeekFrom;
use std::mem;
use time::Date;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

pub const EXE: &str = "exe";
pub const MSI: &str = "msi";
pub const MSIX: &str = "msix";
pub const APPX: &str = "appx";
pub const MSIX_BUNDLE: &str = "msixbundle";
pub const APPX_BUNDLE: &str = "appxbundle";
pub const ZIP: &str = "zip";

pub struct FileAnalyser {
    pub platform: Option<BTreeSet<Platform>>,
    pub minimum_os_version: Option<MinimumOSVersion>,
    pub architecture: Architecture,
    pub installer_type: InstallerType,
    pub installer_sha_256: String,
    pub signature_sha_256: Option<String>,
    pub package_family_name: Option<String>,
    pub product_code: Option<String>,
    pub product_language: Option<LanguageTag>,
    pub last_modified: Option<Date>,
    pub file_name: String,
    pub copyright: Option<Copyright>,
    pub package_name: Option<PackageName>,
    pub publisher: Option<Publisher>,
    pub msi: Option<Msi>,
    pub zip: Option<Zip>,
}

impl FileAnalyser {
    #[async_recursion]
    pub async fn new(file: &mut TempFile, nested: bool) -> Result<Self> {
        let path = file.file_path();
        let file_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .map(str::to_owned)
            .unwrap();
        let extension = path
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_lowercase();
        let pe = (extension == EXE)
            .then(|| VecPE::from_disk_file(path))
            .transpose()?;
        let mut msi = (extension == MSI).then(|| Msi::new(path)).transpose()?;
        let mut msix = match extension.as_str() {
            MSIX | APPX => Some(Msix::new(file).await?),
            _ => None,
        };
        let mut msix_bundle = match extension.as_str() {
            MSIX_BUNDLE | APPX_BUNDLE => Some(MsixBundle::new(file).await?),
            _ => None,
        };
        let zip = if nested {
            None
        } else {
            // File Analyser can be called from within a zip making this function asynchronously recursive
            match extension.as_str() {
                ZIP => Some(Zip::new(file).await?),
                _ => None,
            }
        };
        let installer_type =
            InstallerType::get(file, &extension, msi.as_ref(), pe.as_ref()).await?;
        if installer_type == InstallerType::Burn {
            if let Some(pe) = &pe {
                msi = Some(extract_msi(file, pe).await?);
            }
        }
        let mut string_map = pe
            .as_ref()
            .and_then(|pe| VSVersionInfo::parse(pe).ok())
            .and_then(|vs_version_info| vs_version_info.string_file_info)
            .and_then(|mut info| info.children.swap_remove(0).string_map().ok());
        Ok(Self {
            platform: msix
                .as_ref()
                .map(|msix| BTreeSet::from([msix.target_device_family])),
            minimum_os_version: msix.as_mut().map(|msix| mem::take(&mut msix.min_version)),
            architecture: msi
                .as_ref()
                .map(|msi| msi.architecture)
                .or_else(|| msix.as_ref().map(|msix| msix.processor_architecture))
                .unwrap_or_else(|| Architecture::get_from_exe(&pe.unwrap()).unwrap()),
            installer_type,
            installer_sha_256: String::new(),
            signature_sha_256: msix
                .as_mut()
                .map(|msix| mem::take(&mut msix.signature_sha_256))
                .or_else(|| {
                    msix_bundle
                        .as_mut()
                        .map(|msix_bundle| mem::take(&mut msix_bundle.signature_sha_256))
                }),
            package_family_name: msix
                .map(|msix| msix.package_family_name)
                .or_else(|| msix_bundle.map(|msix_bundle| msix_bundle.package_family_name)),
            product_code: msi.as_mut().map(|msi| mem::take(&mut msi.product_code)),
            product_language: msi.as_mut().map(|msi| mem::take(&mut msi.product_language)),
            last_modified: None,
            file_name,
            copyright: string_map.as_mut().and_then(Copyright::get_from_exe),
            package_name: string_map.as_mut().and_then(PackageName::get_from_exe),
            publisher: string_map.as_mut().and_then(Publisher::get_from_exe),
            msi,
            zip,
        })
    }
}

async fn extract_msi(file: &mut TempFile, pe: &VecPE) -> Result<Msi> {
    let msi_entry = ResourceDirectory::parse(pe)?
        .resources
        .into_iter()
        .find(|entry| entry.rsrc_id == Name(MSI.to_ascii_uppercase()))
        .unwrap()
        .get_data_entry(pe)?;
    let msi_name = format!(
        "{}.{}",
        file.file_path()
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default(),
        MSI
    );
    let mut extracted_msi = TempFile::new_with_name(msi_name).await?.open_rw().await?;
    // Translate the offset into a usable one
    let offset = pe.translate(PETranslation::Memory(msi_entry.offset_to_data))?;

    // Seek to the MSI offset
    file.seek(SeekFrom::Start(offset as u64)).await?;

    // Asynchronously write the MSI to the temporary MSI file
    let mut take = file.take(u64::from(msi_entry.size));
    io::copy(&mut take, &mut extracted_msi).await?;

    let msi = Msi::new(extracted_msi.file_path())?;
    Ok(msi)
}
