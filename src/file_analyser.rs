use crate::exe::vs_version_info::VSVersionInfo;
use crate::manifests::installer_manifest::{Platform, Scope};
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
use camino::Utf8Path;
use chrono::NaiveDate;
use color_eyre::eyre::{Error, OptionExt, Result};
use object::pe::{ImageNtHeaders64, ImageResourceDirectoryEntry, RT_RCDATA};
use object::read::pe::{
    ImageNtHeaders, PeFile, PeFile32, PeFile64, ResourceDirectory, ResourceDirectoryEntryData,
};
use object::{FileKind, LittleEndian, ReadCache, ReadRef};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::io::{Cursor, Read, Seek};
use std::mem;

pub const EXE: &str = "exe";
pub const MSI: &str = "msi";
pub const MSIX: &str = "msix";
pub const APPX: &str = "appx";
pub const MSIX_BUNDLE: &str = "msixbundle";
pub const APPX_BUNDLE: &str = "appxbundle";
pub const ZIP: &str = "zip";

pub struct FileAnalyser<'a> {
    pub platform: Option<BTreeSet<Platform>>,
    pub minimum_os_version: Option<MinimumOSVersion>,
    pub architecture: Option<Architecture>,
    pub installer_type: InstallerType,
    pub scope: Option<Scope>,
    pub installer_sha_256: String,
    pub signature_sha_256: Option<String>,
    pub package_family_name: Option<String>,
    pub product_code: Option<String>,
    pub product_language: Option<LanguageTag>,
    pub last_modified: Option<NaiveDate>,
    pub file_name: Cow<'a, str>,
    pub copyright: Option<Copyright>,
    pub package_name: Option<PackageName>,
    pub publisher: Option<Publisher>,
    pub msi: Option<Msi>,
    pub zip: Option<Zip>,
}

impl<'a> FileAnalyser<'a> {
    pub fn new<R: Read + Seek>(mut reader: R, file_name: Cow<'a, str>) -> Result<Self> {
        let extension = Utf8Path::new(file_name.as_ref())
            .extension()
            .unwrap_or_default()
            .to_lowercase();
        let mut installer_type = None;
        let mut msi = match extension.as_str() {
            MSI => Some(Msi::new(&mut reader)?),
            _ => None,
        };
        let mut pe_arch = None;
        let mut string_map = None;
        let read_ref = ReadCache::new(&mut reader);
        match (extension == EXE)
            .then(|| FileKind::parse(&read_ref).ok())
            .flatten()
        {
            Some(FileKind::Pe32) => {
                let pe_file = PeFile32::parse(&read_ref)?;
                installer_type = Some(InstallerType::get(
                    Some(&pe_file),
                    &extension,
                    msi.as_ref(),
                )?);
                if let Ok((msi_resource, resource_directory)) = get_msi_resource(&pe_file) {
                    installer_type = Some(InstallerType::Burn);
                    msi = Some(extract_msi(&pe_file, msi_resource, resource_directory)?);
                }
                pe_arch = Some(Architecture::get_from_exe(&pe_file)?);
                string_map = VSVersionInfo::parse(&pe_file)
                    .ok()
                    .and_then(|vs_version_info| vs_version_info.string_file_info)
                    .map(|mut string_file_info| {
                        string_file_info.children.swap_remove(0).string_map()
                    });
            }
            Some(FileKind::Pe64) => {
                let pe_file = PeFile64::parse(&read_ref)?;
                installer_type = Some(InstallerType::get(
                    Some(&pe_file),
                    &extension,
                    msi.as_ref(),
                )?);
                if let Ok((msi_resource, resource_directory)) = get_msi_resource(&pe_file) {
                    installer_type = Some(InstallerType::Burn);
                    msi = Some(extract_msi(&pe_file, msi_resource, resource_directory)?);
                }
                pe_arch = Some(Architecture::get_from_exe(&pe_file)?);
                string_map = VSVersionInfo::parse(&pe_file)
                    .ok()
                    .and_then(|vs_version_info| vs_version_info.string_file_info)
                    .map(|mut string_file_info| {
                        string_file_info.children.swap_remove(0).string_map()
                    });
            }
            _ => {}
        }
        let mut msix = match extension.as_str() {
            MSIX | APPX => Some(Msix::new(&mut reader)?),
            _ => None,
        };
        let mut msix_bundle = match extension.as_str() {
            MSIX_BUNDLE | APPX_BUNDLE => Some(MsixBundle::new(&mut reader)?),
            _ => None,
        };
        let mut zip = match extension.as_str() {
            ZIP => Some(Zip::new(reader)?),
            _ => None,
        };
        if installer_type.is_none() {
            installer_type = Some(InstallerType::get::<ImageNtHeaders64, &[u8]>(
                None::<&PeFile<'_, ImageNtHeaders64, &[u8]>>,
                &extension,
                msi.as_ref(),
            )?);
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
                .or_else(|| {
                    msix_bundle.as_ref().and_then(|bundle| {
                        bundle
                            .packages
                            .iter()
                            .find_map(|package| package.processor_architecture)
                    })
                })
                .or(pe_arch)
                .or_else(|| {
                    zip.as_mut()
                        .and_then(|zip| mem::take(&mut zip.architecture))
                }),
            installer_type: installer_type.unwrap(),
            scope: msi.as_ref().and_then(|msi| msi.all_users),
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

fn get_msi_resource<'data, Pe, R>(
    pe: &PeFile<'data, Pe, R>,
) -> Result<(&'data ImageResourceDirectoryEntry, ResourceDirectory<'data>)>
where
    Pe: ImageNtHeaders,
    R: ReadRef<'data>,
{
    let resource_directory = pe
        .data_directories()
        .resource_directory(pe.data(), &pe.section_table())?
        .ok_or_eyre("No resource directory was found")?;

    let rc_data = resource_directory
        .root()?
        .entries
        .iter()
        .find(|entry| entry.name_or_id().id() == Some(RT_RCDATA))
        .ok_or_eyre("No RT_RCDATA was found")?;

    let msi_resource = rc_data.data(resource_directory)?.table().and_then(|table| {
        table.entries.iter().find(|entry| {
            entry
                .name_or_id()
                .name()
                .and_then(|name| name.to_string_lossy(resource_directory).ok())
                .as_deref()
                == Some("MSI")
        })
    });

    msi_resource.map_or_else(
        || Err(Error::msg("No MSI resource was found")),
        |entry| Ok((entry, resource_directory)),
    )
}

pub fn extract_msi<'data, Pe, R>(
    pe: &PeFile<'data, Pe, R>,
    msi_resource: &'data ImageResourceDirectoryEntry,
    resource_directory: ResourceDirectory,
) -> Result<Msi>
where
    Pe: ImageNtHeaders,
    R: ReadRef<'data>,
{
    let msi_entry = msi_resource
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

    // Translate the offset into a usable one
    let offset = {
        let mut rva = msi_entry.offset_to_data.get(LittleEndian);
        rva -= section.virtual_address.get(LittleEndian);
        rva += section.pointer_to_raw_data.get(LittleEndian);
        rva as usize
    };

    // Get the slice that represents the embedded MSI
    let msi_data = Cursor::new(
        pe.data()
            .read_bytes_at(offset as u64, u64::from(msi_entry.size.get(LittleEndian)))
            .unwrap(),
    );

    let msi = Msi::new(msi_data)?;
    Ok(msi)
}
