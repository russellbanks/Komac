use crate::installer_manifest::{Architecture, InstallerType, UpgradeBehavior};
use crate::msi::Msi;
use crate::msix_family::msix::Msix;
use crate::msix_family::msixbundle::MsixBundle;
use async_tempfile::TempFile;
use color_eyre::eyre::{bail, Result};
use exe::ResolvedDirectoryID::Name;
use exe::{CCharString, NTHeaders, PETranslation, ResourceDirectory, VecPE, PE};
use std::ffi::OsStr;
use std::io::SeekFrom;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

const NULLSOFT_BYTES_LEN: usize = 224;

/// The first 224 bytes of an exe made with NSIS are always the same
const NULLSOFT_BYTES: [u8; NULLSOFT_BYTES_LEN] = [
    77, 90, 144, 0, 3, 0, 0, 0, 4, 0, 0, 0, 255, 255, 0, 0, 184, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    216, 0, 0, 0, 14, 31, 186, 14, 0, 180, 9, 205, 33, 184, 1, 76, 205, 33, 84, 104, 105, 115, 32,
    112, 114, 111, 103, 114, 97, 109, 32, 99, 97, 110, 110, 111, 116, 32, 98, 101, 32, 114, 117,
    110, 32, 105, 110, 32, 68, 79, 83, 32, 109, 111, 100, 101, 46, 13, 13, 10, 36, 0, 0, 0, 0, 0,
    0, 0, 173, 49, 8, 129, 233, 80, 102, 210, 233, 80, 102, 210, 233, 80, 102, 210, 42, 95, 57,
    210, 235, 80, 102, 210, 233, 80, 103, 210, 76, 80, 102, 210, 42, 95, 59, 210, 230, 80, 102,
    210, 189, 115, 86, 210, 227, 80, 102, 210, 46, 86, 96, 210, 232, 80, 102, 210, 82, 105, 99,
    104, 233, 80, 102, 210, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    80, 69, 0, 0, 76, 1, 5, 0,
];

const INNO_BYTES_LEN: usize = 264;

/// The first 264 bytes of an exe made with Inno Setup are always the same
const INNO_BYTES: [u8; INNO_BYTES_LEN] = [
    77, 90, 80, 0, 2, 0, 0, 0, 4, 0, 15, 0, 255, 255, 0, 0, 184, 0, 0, 0, 0, 0, 0, 0, 64, 0, 26, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 0, 186, 16, 0, 14, 31, 180, 9, 205, 33, 184, 1, 76, 205, 33, 144, 144, 84, 104, 105,
    115, 32, 112, 114, 111, 103, 114, 97, 109, 32, 109, 117, 115, 116, 32, 98, 101, 32, 114, 117,
    110, 32, 117, 110, 100, 101, 114, 32, 87, 105, 110, 51, 50, 13, 10, 36, 55, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 80, 69, 0, 0, 76, 1, 10, 0,
];

const EXE: &str = "exe";
const MSI: &str = "msi";
const MSIX: &str = "msix";
const APPX: &str = "appx";
const MSIX_BUNDLE: &str = "msixbundle";
const APPX_BUNDLE: &str = "appxbundle";
const ZIP: &str = "zip";

pub struct FileAnalyser {
    pub installer_type: InstallerType,
    pub msi: Option<Msi>,
    pub msix: Option<Msix>,
    pub msix_bundle: Option<MsixBundle>,
    pub pe: Option<VecPE>,
}

impl FileAnalyser {
    pub async fn new(file: &mut TempFile) -> Result<FileAnalyser> {
        let path = file.file_path();
        let extension = path
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_lowercase();
        let pe = (extension == EXE)
            .then(|| VecPE::from_disk_file(path))
            .transpose()?;
        let mut msi = (extension == MSI).then(|| Msi::new(path)).transpose()?;
        let msix = match extension.as_str() {
            MSIX | APPX => Some(Msix::new(file).await),
            _ => None,
        }
        .transpose()?;
        let msix_bundle = match extension.as_str() {
            MSIX_BUNDLE | APPX_BUNDLE => Some(MsixBundle::new(file).await),
            _ => None,
        }
        .transpose()?;
        let installer_type = get_installer_type(file, &extension, &msi, &pe).await?;
        if installer_type == InstallerType::Burn {
            if let Some(ref pe) = pe {
                msi = Some(extract_msi(file, pe).await?);
            }
        }
        Ok(FileAnalyser {
            installer_type,
            msi,
            msix,
            msix_bundle,
            pe,
        })
    }
}

async fn extract_msi(file: &mut TempFile, pe: &VecPE) -> Result<Msi> {
    let msi_entry = ResourceDirectory::parse(pe)?
        .resources
        .into_iter()
        .find(|entry| entry.rsrc_id == Name(MSI.to_uppercase()))
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
    let mut take = file.take(msi_entry.size as u64);
    io::copy(&mut take, &mut extracted_msi).await?;

    let msi = Msi::new(extracted_msi.file_path())?;
    Ok(msi)
}

pub fn get_architecture(pe: &VecPE) -> Result<Architecture> {
    let machine = match pe.get_valid_nt_headers()? {
        NTHeaders::NTHeaders32(nt_header) => nt_header.file_header.machine,
        NTHeaders::NTHeaders64(nt_header) => nt_header.file_header.machine,
    };
    // https://learn.microsoft.com/windows/win32/debug/pe-format#machine-types
    Ok(match machine {
        34404 => Architecture::X64,           // 0x8664
        332 => Architecture::X86,             // 0x14c
        43620 => Architecture::Arm64,         // 0xaa64
        448 | 450 | 452 => Architecture::Arm, // 0x1c0 | 0x1c2 | 0x1c4
        0 => Architecture::Neutral,           // 0x0
        _ => bail!("Unknown machine value {:04x}", machine),
    })
}

async fn get_installer_type(
    file: &mut TempFile,
    extension: &str,
    msi: &Option<Msi>,
    pe: &Option<VecPE>,
) -> Result<InstallerType> {
    match extension {
        MSI => {
            if let Some(msi) = msi {
                return Ok(match msi.is_wix {
                    true => InstallerType::Wix,
                    false => InstallerType::Msi,
                });
            }
        }
        MSIX | MSIX_BUNDLE => return Ok(InstallerType::Msix),
        APPX | APPX_BUNDLE => return Ok(InstallerType::Appx),
        ZIP => return Ok(InstallerType::Zip),
        EXE => {
            // Check if the file is Inno or Nullsoft from their magic bytes
            let mut buffer = [0; INNO_BYTES_LEN];
            file.seek(SeekFrom::Start(0)).await?;
            file.read_exact(&mut buffer).await?;
            match () {
                _ if buffer == INNO_BYTES => return Ok(InstallerType::Inno),
                _ if buffer[..NULLSOFT_BYTES_LEN] == NULLSOFT_BYTES => {
                    return Ok(InstallerType::Nullsoft)
                }
                _ => {}
            };
            if let Some(pe) = pe {
                match () {
                    _ if has_msi_resource(pe) => return Ok(InstallerType::Burn),
                    _ if has_burn_header(pe) => return Ok(InstallerType::Burn),
                    _ => {}
                }
            }
            return Ok(InstallerType::Exe);
        }
        _ => {}
    }
    bail!("Unsupported file extension {extension}")
}

fn has_msi_resource(pe: &VecPE) -> bool {
    ResourceDirectory::parse(pe)
        .map(|resource_directory| {
            resource_directory
                .resources
                .iter()
                .any(|entry| match &entry.rsrc_id {
                    Name(value) => value.to_lowercase() == MSI,
                    _ => false,
                })
        })
        .unwrap_or(false)
}

fn has_burn_header(pe: &VecPE) -> bool {
    const WIX_BURN_HEADER: &str = ".wixburn";

    pe.get_section_table()
        .map(|section_table| {
            section_table
                .iter()
                .any(|section| section.name.as_str().unwrap_or_default() == WIX_BURN_HEADER)
        })
        .unwrap_or(false)
}

pub fn get_upgrade_behavior(installer_type: &InstallerType) -> Option<UpgradeBehavior> {
    match installer_type {
        InstallerType::Msix | InstallerType::Appx => Some(UpgradeBehavior::Install),
        _ => None,
    }
}
