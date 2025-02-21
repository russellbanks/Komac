use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    installer::{installer_return_code::InstallerReturnCode, return_response::ReturnResponse},
    shared::url::DecodedUrl,
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct ExpectedReturnCodes {
    pub installer_return_code: Option<InstallerReturnCode>,
    pub return_response: ReturnResponse,
    pub return_response_url: Option<DecodedUrl>,
}
