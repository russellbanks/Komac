use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

use crate::shared::url::DecodedUrl;

#[derive(Clone, FromStr, Default, Display, Deserialize, Serialize)]
pub struct PublisherUrl(DecodedUrl);
