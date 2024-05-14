use color_eyre::eyre::{bail, Result};
use inquire::error::InquireResult;
use inquire::validator::Validation;
use inquire::Password;
use keyring::Entry;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, DNT, USER_AGENT};
use reqwest::{Client, StatusCode};
use tokio::runtime::Handle;

const SERVICE: &str = "komac";
const USERNAME: &str = "github-access-token";
const GITHUB_API_ENDPOINT: &str = "https://api.github.com/octocat";

pub fn get_komac_credential() -> keyring::Result<Entry> {
    Entry::new(SERVICE, USERNAME)
}

pub async fn handle_token(token: Option<String>) -> Result<String> {
    let client = Client::builder()
        .default_headers(get_default_headers(None))
        .build()?;

    if let Some(token) = token {
        return validate_token(&client, &token).await.map(|()| token);
    }

    let credential_entry = get_komac_credential()?;

    if let Ok(stored_token) = credential_entry.get_password() {
        validate_token(&client, &stored_token)
            .await
            .map(|()| stored_token)
    } else {
        let token = token_prompt(client, None)?;
        if credential_entry.set_password(&token).is_ok() {
            println!("Successfully stored token in platform's secure storage");
        }
        Ok(token)
    }
}

pub fn token_prompt(client: Client, prompt: Option<&str>) -> InquireResult<String> {
    tokio::task::block_in_place(|| {
        let rt = Handle::current();
        let validator =
            move |input: &str| match rt.block_on(async { validate_token(&client, input).await }) {
                Ok(()) => Ok(Validation::Valid),
                Err(err) => Ok(Validation::Invalid(err.into())),
            };
        Password::new(prompt.unwrap_or("Enter a GitHub token"))
            .with_validator(validator)
            .without_confirmation()
            .prompt()
    })
}

pub async fn validate_token(client: &Client, token: &str) -> Result<()> {
    match client
        .get(GITHUB_API_ENDPOINT)
        .bearer_auth(token)
        .send()
        .await?
        .status()
    {
        StatusCode::UNAUTHORIZED => bail!("GitHub token is invalid"),
        _ => Ok(()),
    }
}

#[allow(clippy::declare_interior_mutable_const)]
const MICROSOFT_DELIVERY_OPTIMIZATION: HeaderValue =
    HeaderValue::from_static("Microsoft-Delivery-Optimization/10.1");
const SEC_GPC: &str = "Sec-GPC";

pub fn get_default_headers(github_token: Option<&str>) -> HeaderMap {
    let mut default_headers = HeaderMap::new();
    default_headers.insert(USER_AGENT, MICROSOFT_DELIVERY_OPTIMIZATION);
    default_headers.insert(DNT, HeaderValue::from(1));
    default_headers.insert(SEC_GPC, HeaderValue::from(1));
    if let Some(token) = github_token {
        if let Ok(bearer_auth) = HeaderValue::from_str(&format!("Bearer {token}")) {
            default_headers.insert(AUTHORIZATION, bearer_auth);
        }
    }
    default_headers
}
