use crate::manifests::installer_manifest::Scope;
use crate::types::architecture::Architecture;
use crate::types::language_tag::LanguageTag;
use color_eyre::eyre::{bail, Result};
use msi::{Language, Package, Select};
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::str::FromStr;

pub struct Msi {
    pub architecture: Architecture,
    pub product_code: String,
    pub upgrade_code: String,
    pub product_name: String,
    pub product_version: String,
    pub manufacturer: String,
    pub product_language: LanguageTag,
    pub all_users: Option<Scope>,
    pub is_wix: bool,
}

const PROPERTY: &str = "Property";
const PRODUCT_CODE: &str = "ProductCode";
const PRODUCT_LANGUAGE: &str = "ProductLanguage";
const PRODUCT_NAME: &str = "ProductName";
const PRODUCT_VERSION: &str = "ProductVersion";
const MANUFACTURER: &str = "Manufacturer";
const UPGRADE_CODE: &str = "UpgradeCode";
const ALL_USERS: &str = "ALLUSERS";
const WIX: &str = "wix";

impl Msi {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut msi = Package::open(reader)?;

        let architecture = match msi.summary_info().arch() {
            Some("x64" | "Intel64" | "AMD64") => Architecture::X64,
            Some("Intel") => Architecture::X86,
            Some("Arm64") => Architecture::Arm64,
            Some("Arm") => Architecture::Arm,
            _ => bail!("No architecture was found in the MSI"),
        };

        let mut property_map = msi
            .select_rows(Select::table(PROPERTY))?
            .filter_map(|row| {
                if row.len() == 2 {
                    // Property and Value column
                    if let (Some(property), Some(value)) = (row[0].as_str(), row[1].as_str()) {
                        return Some((property.to_owned(), value.to_owned()));
                    }
                }
                None
            })
            .collect::<HashMap<_, _>>();

        Ok(Self {
            architecture,
            product_code: property_map.remove(PRODUCT_CODE).unwrap(),
            upgrade_code: property_map.remove(UPGRADE_CODE).unwrap(),
            product_name: property_map.remove(PRODUCT_NAME).unwrap(),
            product_version: property_map.remove(PRODUCT_VERSION).unwrap(),
            manufacturer: property_map.remove(MANUFACTURER).unwrap(),
            product_language: LanguageTag::from_str(
                Language::from_code(u16::from_str(property_map.get(PRODUCT_LANGUAGE).unwrap())?)
                    .tag(),
            )?,
            // https://learn.microsoft.com/windows/win32/msi/allusers
            all_users: match property_map.remove(ALL_USERS).unwrap_or_default().as_str() {
                "1" => Some(Scope::Machine),
                "2" => None, // Installs depending on installation context and user privileges
                _ => Some(Scope::User), // No value or an empty string specifies per-user context
            },
            is_wix: property_map.into_iter().any(|(mut property, mut value)| {
                property.make_ascii_lowercase();
                property.contains(WIX) || {
                    value.make_ascii_lowercase();
                    value.contains(WIX)
                }
            }),
        })
    }
}
