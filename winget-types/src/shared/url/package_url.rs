use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

use crate::shared::url::DecodedUrl;

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct PackageUrl(DecodedUrl);
