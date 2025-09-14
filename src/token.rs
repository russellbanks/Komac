use std::borrow::Cow;

use bon::bon;
use color_eyre::eyre::Result;
use inquire::{InquireError, Password, error::InquireResult, validator::Validation};
use keyring::Entry;
use reqwest::{
    Client, StatusCode,
    header::{AUTHORIZATION, DNT, HeaderMap, HeaderValue, USER_AGENT},
};
use thiserror::Error;
use tokio::runtime::Handle;

use crate::{commands::utils::environment::CI, prompts::handle_inquire_error};

const SERVICE: &str = "komac";
const USERNAME: &str = "github-access-token";
const GITHUB_API_ENDPOINT: &str = "https://api.github.com/octocat";

#[derive(Debug, Error)]
pub enum TokenError {
    #[error(
        "No token was provided or stored. Provide one with the `GITHUB_TOKEN` environment variable or `--token`."
    )]
    NoTokenInCI,
    #[error("GitHub token is invalid.")]
    InvalidToken,
    #[error("Failed to connect to GitHub. Please check your internet connection.")]
    FailedToConnect,
    #[error(transparent)]
    Keyring(#[from] keyring::Error),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    Inquire(#[from] InquireError),
}

pub struct TokenManager<'a> {
    token: Cow<'a, str>,
}

#[bon]
impl<'a> TokenManager<'a> {
    pub async fn handle<T>(token: Option<T>) -> Result<Self, TokenError>
    where
        T: Into<Cow<'a, str>>,
    {
        // Token rules:
        // - If caller passed `--token`: validate it and fail if invalid.
        // - Otherwise try keyring:
        //     * In CI: if no token or if stored token is invalid -> error (never prompt).
        //     * Interactive: if no stored token or stored token is invalid -> prompt and store.

        let client = Client::builder()
            .default_headers(default_headers(None))
            .build()?;

        let credential = Self::credential()?;

        let token_passed = token.is_some();

        let token = if let Some(token) = token.map(T::into) {
            Some(token)
        } else {
            match credential.get_password() {
                Ok(token) => Some(Cow::Owned(token)),
                Err(keyring::Error::NoEntry) if *CI => return Err(TokenError::NoTokenInCI),
                Err(keyring::Error::NoEntry) => None, // No stored token, must prompt
                Err(error) => return Err(TokenError::Keyring(error)),
            }
        };

        if let Some(token) = token {
            match Self::validate(&client, &token).await {
                Ok(()) => return Ok(Self { token }),
                Err(TokenError::InvalidToken) if token_passed || *CI => {
                    return Err(TokenError::InvalidToken);
                }
                Err(TokenError::InvalidToken) => {}
                Err(err) => return Err(err),
            }
        }

        let validated_token = Self::prompt().client(&client).call()?;

        if credential.set_password(&validated_token).is_ok() {
            println!("Successfully stored token in platform's secure storage");
        }

        Ok(Self {
            token: Cow::Owned(validated_token),
        })
    }

    #[builder]
    pub fn prompt(
        client: &Client,
        #[builder(default = "Enter a GitHub token")] message: &str,
    ) -> InquireResult<String> {
        tokio::task::block_in_place(|| {
            let rt = Handle::current();
            let client = client.clone();
            let validator = move |input: &str| match rt
                .block_on(async { Self::validate(&client, input).await })
            {
                Ok(()) => Ok(Validation::Valid),
                Err(err) => Ok(Validation::Invalid(err.into())),
            };

            Password::new(message)
                .with_validator(validator)
                .without_confirmation()
                .prompt()
                .map_err(handle_inquire_error)
        })
    }

    pub async fn validate(client: &Client, token: &str) -> Result<(), TokenError> {
        match client
            .get(GITHUB_API_ENDPOINT)
            .bearer_auth(token)
            .send()
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::UNAUTHORIZED => Err(TokenError::InvalidToken),
                _ => Ok(()),
            },
            Err(error) => {
                if error.is_connect() {
                    Err(TokenError::FailedToConnect)
                } else {
                    Err(error.into())
                }
            }
        }
    }

    #[inline]
    pub fn credential() -> keyring::Result<Entry> {
        Entry::new(SERVICE, USERNAME)
    }
}

impl AsRef<str> for TokenManager<'_> {
    fn as_ref(&self) -> &str {
        self.token.as_ref()
    }
}

const MICROSOFT_DELIVERY_OPTIMIZATION: HeaderValue =
    HeaderValue::from_static("Microsoft-Delivery-Optimization/10.1");
const SEC_GPC: &str = "Sec-GPC";

pub fn default_headers(github_token: Option<&str>) -> HeaderMap {
    let mut default_headers = HeaderMap::new();
    default_headers.insert(USER_AGENT, MICROSOFT_DELIVERY_OPTIMIZATION);
    default_headers.insert(DNT, HeaderValue::from(1));
    default_headers.insert(SEC_GPC, HeaderValue::from(1));
    if let Some(token) = github_token
        && let Ok(bearer_auth) = HeaderValue::from_str(&format!("Bearer {token}"))
    {
        default_headers.insert(AUTHORIZATION, bearer_auth);
    }
    default_headers
}
