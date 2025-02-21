use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Agreement {
    #[serde(rename = "AgreementLabel")]
    pub label: Option<String>,
    #[serde(rename = "Agreement")]
    pub text: Option<String>,
    #[serde(rename = "AgreementUrl")]
    pub url: Option<Url>,
}
