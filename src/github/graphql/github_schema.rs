use cynic::impl_scalar;
use url::Url;

#[cynic::schema("github")]
pub mod github_schema {}

impl_scalar!(Url, github_schema::URI);
