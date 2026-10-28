use thiserror::Error;

#[derive(Debug, Error)]
pub enum MoonbridgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid DAP message: {0}")]
    InvalidMessage(String),

    #[error("Channel error: {0}")]
    Channel(String),
}

pub type Result<T> = std::result::Result<T, MoonbridgeError>;
