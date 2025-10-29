use thiserror::Error;

/// Errors that can occur while using the Dlsite API
#[derive(Debug, Error)]
pub enum DlsiteError {
    /// HTTP request error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// HTTP status code error
    #[error("HTTP error: {0}")]
    HttpStatus(u16),

    /// Rate limit error - too many requests
    #[error("Rate limited: {0}")]
    RateLimit(String),

    /// Request timeout error
    #[error("Request timeout")]
    Timeout,

    /// HTML/JSON parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Server-side error
    #[error("Server error: {0}")]
    Server(String),
}

pub(crate) type Result<T> = std::result::Result<T, DlsiteError>;
