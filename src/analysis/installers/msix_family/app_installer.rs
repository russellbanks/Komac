use color_eyre::{Result, eyre::eyre};
use quick_xml::de::from_str;
use reqwest::Response;
use serde::Deserialize;
use tracing::debug;
use url::Url;

pub struct AppInstaller;

impl AppInstaller {
    /// Returns the main package's URL from the `AppInstaller`'s manifest.
    pub async fn fetch_main_url(response: Response) -> Result<Url> {
        let manifest = response.text().await?;
        debug!("{manifest:#?}");

        let manifest = from_str::<Manifest>(&manifest)?;
        debug!(?manifest);

        if let Some(main) = manifest.main() {
            Ok(main.uri)
        } else {
            Err(eyre!(
                "AppInstaller manifest contains no MainBundle or MainPackage"
            ))
        }
    }
}

/// <https://learn.microsoft.com/uwp/schemas/appinstallerschema/element-appinstaller>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename = "AppInstaller", rename_all = "PascalCase")]
struct Manifest {
    main_bundle: Option<Main>,
    main_package: Option<Main>,
}

impl Manifest {
    /// Returns the main bundle, or the main package if the main bundle is not present.
    fn main(self) -> Option<Main> {
        self.main_bundle.or(self.main_package)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Main {
    /// A string between 3 and 50 characters in length that consists of
    /// alphanumeric, period, and dash characters.
    #[serde(rename = "@Name")]
    name: String,

    #[serde(rename = "@Publisher")]
    publisher: String,

    /// A version string in quad notation, "Major.Minor.Build.Revision"
    #[serde(rename = "@Version")]
    version: String,

    /// Uri to the app package location
    #[serde(rename = "@Uri")]
    uri: Url,
}
