use crate::exe::vs_version_info::VSVersionInfo;
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
use color_eyre::eyre::{OptionExt, Result};
use memmap2::Mmap;
use object::pe::{ImageNtHeaders64, RT_RCDATA};
use object::read::pe::{ImageNtHeaders, PeFile, PeFile32, PeFile64, ResourceDirectoryEntryData};
use object::{FileKind, LittleEndian, ReadRef};
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
        let mut msi = (extension == MSI).then(|| Msi::new(path)).transpose()?;
        let mut installer_type = None;
        let mmap_file = std::fs::File::open(path)?;
        let map = unsafe { Mmap::map(&mmap_file) }?;
        let mut pe_arch = None;
        let mut string_map = None;
        match FileKind::parse(map.as_ref())? {
            FileKind::Pe32 => {
                let pe_file = PeFile32::parse(map.as_ref())?;
                installer_type = Some(
                    InstallerType::get(map.as_ref(), Some(&pe_file), &extension, msi.as_ref())
                        .await?,
                );
                if installer_type == Some(InstallerType::Burn) {
                    msi = Some(extract_msi(file, &pe_file).await?);
                }
                pe_arch = Some(Architecture::get_from_exe(&pe_file)?);
                string_map = VSVersionInfo::parse(&pe_file, map.as_ref())?
                    .string_file_info
                    .map(|mut string_file_info| {
                        string_file_info.children.swap_remove(0).string_map()
                    })
            }
            FileKind::Pe64 => {
                let pe_file = PeFile64::parse(map.as_ref())?;
                installer_type = Some(
                    InstallerType::get(map.as_ref(), Some(&pe_file), &extension, msi.as_ref())
                        .await?,
                );
                if installer_type == Some(InstallerType::Burn) {
                    msi = Some(extract_msi(file, &pe_file).await?);
                }
                pe_arch = Some(Architecture::get_from_exe(&pe_file)?);
                string_map = VSVersionInfo::parse(&pe_file, map.as_ref())?
                    .string_file_info
                    .map(|mut string_file_info| {
                        string_file_info.children.swap_remove(0).string_map()
                    })
            }
            _ => {}
        }
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
        if installer_type.is_none() {
            installer_type = Some(
                InstallerType::get::<ImageNtHeaders64, &[u8]>(
                    map.as_ref(),
                    None::<&PeFile<'_, ImageNtHeaders64, &[u8]>>,
                    &extension,
                    msi.as_ref(),
                )
                .await?,
            );
        }
        Ok(Self {
            platform: msix
                .as_ref()
                .map(|msix| BTreeSet::from([msix.target_device_family])),
            minimum_os_version: msix.as_mut().map(|msix| mem::take(&mut msix.min_version)),
            architecture: msi
                .as_ref()
                .map(|msi| msi.architecture)
                .or_else(|| msix.as_ref().map(|msix| msix.processor_architecture))
                .unwrap_or_else(|| pe_arch.unwrap()),
            installer_type: installer_type.unwrap(),
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

pub async fn extract_msi<'data, Pe, R>(
    file: &mut TempFile,
    pe: &PeFile<'data, Pe, R>,
) -> Result<Msi>
where
    Pe: ImageNtHeaders,
    R: ReadRef<'data>,
{
    let resource_directory = pe
        .data_directories()
        .resource_directory(pe.data(), &pe.section_table())?
        .ok_or_eyre("No resource directory")?;
    let rc_data = resource_directory
        .root()?
        .entries
        .iter()
        .find(|entry| entry.name_or_id().id() == Some(RT_RCDATA))
        .ok_or_eyre("No RT_RCDATA was found")?;
    let msi = rc_data
        .data(resource_directory)?
        .table()
        .and_then(|table| {
            table.entries.iter().find(|entry| {
                entry
                    .name_or_id()
                    .name()
                    .and_then(|name| name.to_string_lossy(resource_directory).ok())
                    .as_deref()
                    == Some("MSI")
            })
        })
        .ok_or_eyre("No MSI resource was found")?;
    let msi_entry = msi
        .data(resource_directory)?
        .table()
        .and_then(|table| table.entries.first())
        .and_then(|entry| entry.data(resource_directory).ok())
        .and_then(ResourceDirectoryEntryData::data)
        .unwrap();

    let section = pe
        .section_table()
        .section_containing(msi_entry.offset_to_data.get(LittleEndian))
        .unwrap();

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
    let offset = {
        let mut rva = msi_entry.offset_to_data.get(LittleEndian);
        rva -= section.virtual_address.get(LittleEndian);
        rva += section.pointer_to_raw_data.get(LittleEndian);
        rva
    };

    // Seek to the MSI offset
    file.seek(SeekFrom::Start(u64::from(offset))).await?;

    // Asynchronously write the MSI to the temporary MSI file
    let mut take = file.take(u64::from(msi_entry.size.get(LittleEndian)));
    io::copy(&mut take, &mut extracted_msi).await?;

    let msi = Msi::new(extracted_msi.file_path())?;
    Ok(msi)
}
