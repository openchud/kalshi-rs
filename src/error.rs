/// Errors returned by the Kalshi SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    /// JSON deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// RSA key loading or signing failed.
    #[error("Auth error: {0}")]
    Auth(String),

    /// WebSocket connection or protocol error.
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// I/O error (reading key files, etc).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
