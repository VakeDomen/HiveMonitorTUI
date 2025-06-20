use std::{error::Error as StdError, fmt, io::Error, string::FromUtf8Error};
use reqwest;
use serde_json;
use toml;
use reqwest::header::InvalidHeaderValue;

/// Errors returned by HTTP client operations
#[derive(Debug)]
pub enum ClientError {
    /// HTTP transport or status error
    Reqwest(reqwest::Error),
    /// Invalid header value (e.g., Authorization)
    HeaderInvalid(InvalidHeaderValue),
    /// JSON deserialization error
    Json(serde_json::Error),
    /// TOML parsing or serialization error
    Toml(toml::de::Error),
    /// I/O error (e.g., reading config)
    Io(std::io::Error),
    Decode(FromUtf8Error),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Reqwest(e) => write!(f, "HTTP error: {}", e),
            ClientError::HeaderInvalid(e) => write!(f, "Invalid header: {}", e),
            ClientError::Json(e) => write!(f, "JSON error: {}", e),
            ClientError::Toml(e) => write!(f, "TOML error: {}", e),
            ClientError::Io(e) => write!(f, "I/O error: {}", e),
            ClientError::Decode(e) => write!(f, "Decoding error: {}", e),
                    }
    }
}

impl StdError for ClientError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ClientError::Reqwest(e) => Some(e),
            ClientError::HeaderInvalid(e) => Some(e),
            ClientError::Json(e) => Some(e),
            ClientError::Toml(e) => Some(e),
            ClientError::Io(e) => Some(e),
            ClientError::Decode(e) => Some(e),
                    }
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        ClientError::Reqwest(e)
    }
}

impl From<InvalidHeaderValue> for ClientError {
    fn from(e: InvalidHeaderValue) -> Self {
        ClientError::HeaderInvalid(e)
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> Self {
        ClientError::Json(e)
    }
}

impl From<toml::de::Error> for ClientError {
    fn from(e: toml::de::Error) -> Self {
        ClientError::Toml(e)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::Io(e)
    }
}

pub type ClipboardError = Box<dyn std::error::Error>;

