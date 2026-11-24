use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

/// An unsuccessful response from the GitHub REST API.
#[derive(Debug, Error)]
#[error("GitHub returned {status}: {message}")]
pub struct RestError {
    /// The HTTP status of the response.
    pub status: StatusCode,
    /// GitHub's explanation of the error, or a fallback when the body has none.
    pub message: String,
}

impl RestError {
    /// The most characters of a non-JSON body to keep, so an HTML error page does not flood the
    /// terminal.
    const MAX_BODY_CHARS: usize = 200;

    /// Builds an error from a response's status and body.
    ///
    /// GitHub normally sends a JSON body with a `message`, but proxies and outages can produce
    /// HTML or an empty body, so those fall back to the body text or the status reason.
    pub fn from_body(status: StatusCode, body: &[u8]) -> Self {
        let message = serde_json::from_slice::<ErrorBody>(body)
            .ok()
            .map(|body| body.message)
            .or_else(|| Self::body_text(body))
            .unwrap_or_else(|| {
                status
                    .canonical_reason()
                    .unwrap_or("no response body")
                    .to_owned()
            });

        Self { status, message }
    }

    /// Returns the trimmed body text, shortened to [`Self::MAX_BODY_CHARS`], or `None` if it is
    /// blank.
    fn body_text(body: &[u8]) -> Option<String> {
        let text = String::from_utf8_lossy(body);
        let text = text.trim();

        if text.is_empty() {
            return None;
        }

        match text.char_indices().nth(Self::MAX_BODY_CHARS) {
            Some((cut, _)) => Some(format!("{}…", &text[..cut])),
            None => Some(text.to_owned()),
        }
    }
}

/// The JSON body GitHub sends with an unsuccessful response.
#[derive(Deserialize)]
struct ErrorBody {
    message: String,
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::RestError;

    #[test]
    fn json_body_message() {
        let error = RestError::from_body(
            StatusCode::UNPROCESSABLE_ENTITY,
            br#"{
                "message": "Validation Failed",
                "documentation_url": "https://docs.github.com/rest"
            }"#,
        );

        assert_eq!(error.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error.message, "Validation Failed");
    }

    #[test]
    fn json_body_with_status_field() {
        let error = RestError::from_body(
            StatusCode::UNPROCESSABLE_ENTITY,
            br#"{
                "status": "422",
                "message": "refusing to allow a Personal Access Token to create or update workflow `.github/workflows/foo.yaml` without `workflow` scope"
            }"#,
        );

        assert_eq!(
            error.message,
            "refusing to allow a Personal Access Token to create or update workflow `.github/workflows/foo.yaml` without `workflow` scope"
        );
    }

    #[test]
    fn non_json_body_uses_text() {
        let error = RestError::from_body(StatusCode::BAD_GATEWAY, b"  <h1>Bad gateway</h1>\n");

        assert_eq!(error.status, StatusCode::BAD_GATEWAY);
        assert_eq!(error.message, "<h1>Bad gateway</h1>");
    }

    #[test]
    fn long_non_json_body_is_truncated() {
        let body = "x".repeat(1000);

        let error = RestError::from_body(StatusCode::BAD_GATEWAY, body.as_bytes());

        assert_eq!(
            error.message,
            format!("{}…", "x".repeat(RestError::MAX_BODY_CHARS))
        );
    }

    #[test]
    fn empty_body_uses_status_reason() {
        let error = RestError::from_body(StatusCode::SERVICE_UNAVAILABLE, b"");

        assert_eq!(error.message, "Service Unavailable");
    }

    #[test]
    fn display_includes_status() {
        let error = RestError::from_body(StatusCode::NOT_FOUND, br#"{"message":"Not Found"}"#);

        assert_eq!(
            error.to_string(),
            "GitHub returned 404 Not Found: Not Found"
        );
    }
}
