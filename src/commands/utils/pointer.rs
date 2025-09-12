use std::{str::FromStr, sync::Arc};

use serde::de::StdError;

#[inline]
pub fn arc<T: FromStr>(r#type: &str) -> Result<Arc<T>, <T as FromStr>::Err>
where
    <T as FromStr>::Err: Send + Sync + StdError + 'static,
{
    r#type.parse().map(Arc::new)
}
