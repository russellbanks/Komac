use chrono::{DateTime, Utc};
use cynic::impl_scalar;
use url::Url;

#[cynic::schema("github")]
pub mod github_schema {}

impl_scalar!(Url, github_schema::URI);
impl_scalar!(DateTime<Utc>, github_schema::DateTime);
