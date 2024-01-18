use color_eyre::eyre::bail;
use inquire::validator::Validation;
use inquire::Password;
use keyring::{Entry, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, DNT, USER_AGENT};
use reqwest::{Client, StatusCode};
use tokio::runtime::Handle;

const SERVICE: &str = "komac";
const USERNAME: &str = "github-access-token";
const GITHUB_API_ENDPOINT: &str = "https://api.github.com/octocat";

pub fn get_komac_credential() -> Result<Entry> {
    Entry::new(SERVICE, USERNAME)
}

pub async fn handle_token(token: Option<String>) -> color_eyre::eyre::Result<String> {
    if let Some(token) = token {
        return Ok(token);
    }

    let credential_entry = get_komac_credential()?;

    if let Ok(stored_token) = credential_entry.get_password() {
        let client = Client::builder()
            .default_headers(get_default_headers(Some(&stored_token)))
            .build()?;
        match client.get(GITHUB_API_ENDPOINT).send().await?.status() {
            StatusCode::UNAUTHORIZED => bail!("GitHub token is invalid"),
            _ => Ok(stored_token),
        }
    } else {
        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;
        let token = tokio::task::block_in_place(move || {
            let rt = Handle::current();
            let validator = move |input: &str| match rt
                .block_on(async {
                    client
                        .get(GITHUB_API_ENDPOINT)
                        .bearer_auth(input)
                        .send()
                        .await
                })?
                .status()
            {
                StatusCode::UNAUTHORIZED => {
                    Ok(Validation::Invalid("GitHub token is invalid".into()))
                }
                _ => Ok(Validation::Valid),
            };
            Password::new("Enter a GitHub token")
                .with_validator(validator)
                .without_confirmation()
                .prompt()
        })?;
        if credential_entry.set_password(&token).is_ok() {
            println!("Successfully stored token in platform's secure storage");
        }
        Ok(token)
    }
}

const MICROSOFT_DELIVERY_OPTIMIZATION: HeaderValue =
    HeaderValue::from_static("Microsoft-Delivery-Optimization/10.1");

pub fn get_default_headers(github_token: Option<&str>) -> HeaderMap {
    let mut default_headers = HeaderMap::new();
    default_headers.insert(USER_AGENT, MICROSOFT_DELIVERY_OPTIMIZATION);
    default_headers.insert(DNT, HeaderValue::from(1));
    if let Some(token) = github_token {
        if let Ok(bearer_auth) = HeaderValue::from_str(&format!("Bearer {token}")) {
            default_headers.insert(AUTHORIZATION, bearer_auth);
        }
    }
    default_headers
}
