//! Error types for the RUSSH Client

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application error type
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum AppError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid authentication method")]
    InvalidAuthMethod,

    #[error("Missing profile ID")]
    MissingProfileId,

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("File operation failed: {0}")]
    FileOperationFailed(String),

    #[error("Transfer failed: {0}")]
    TransferFailed(String),

    #[error("P2P connection failed: {0}")]
    P2PConnectionFailed(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Settings error: {0}")]
    SettingsError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerializationError(err.to_string())
    }
}

impl From<russh_ssh::ConnectionError> for AppError {
    fn from(err: russh_ssh::ConnectionError) -> Self {
        AppError::ConnectionFailed(err.to_string())
    }
}

// Make AppError compatible with Tauri's error handling
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", &self.error_code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl AppError {
    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::ConnectionFailed(_) => "CONNECTION_FAILED",
            AppError::AuthenticationFailed(_) => "AUTH_FAILED",
            AppError::SessionNotFound(_) => "SESSION_NOT_FOUND",
            AppError::InvalidAuthMethod => "INVALID_AUTH_METHOD",
            AppError::MissingProfileId => "MISSING_PROFILE_ID",
            AppError::ProfileNotFound(_) => "PROFILE_NOT_FOUND",
            AppError::FileOperationFailed(_) => "FILE_OPERATION_FAILED",
            AppError::TransferFailed(_) => "TRANSFER_FAILED",
            AppError::P2PConnectionFailed(_) => "P2P_CONNECTION_FAILED",
            AppError::PeerNotFound(_) => "PEER_NOT_FOUND",
            AppError::SettingsError(_) => "SETTINGS_ERROR",
            AppError::SerializationError(_) => "SERIALIZATION_ERROR",
            AppError::IoError(_) => "IO_ERROR",
            AppError::InternalError(_) => "INTERNAL_ERROR",
        }
    }
}
