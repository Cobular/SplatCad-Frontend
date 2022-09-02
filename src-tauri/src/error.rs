use std::fmt::Display;

use generic_error::GenericError;
use serde_json::Value;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Generic(GenericError),
    TauriError(tauri::api::Error),
    IoError(std::io::Error),
    WalkDirError(walkdir::Error),
    StringError(String),
    SledError(sled::Error),
    SerdeCborError(serde_cbor::Error),
}

impl From<GenericError> for Error {
    fn from(error: GenericError) -> Self {
        Error::Generic(error)
    }
}

impl From<tauri::api::Error> for Error {
    fn from(error: tauri::api::Error) -> Self {
        Error::TauriError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<walkdir::Error> for Error {
    fn from(error: walkdir::Error) -> Self {
        Error::WalkDirError(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::StringError(error)
    }
}

impl From<sled::Error> for Error {
    fn from(error: sled::Error) -> Self {
        Error::SledError(error)
    }
}

impl From<serde_cbor::Error> for Error {
    fn from(error: serde_cbor::Error) -> Self {
        Error::SerdeCborError(error)
    }
}
impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Generic(error) => write!(f, "{}", error),
            Error::TauriError(error) => write!(f, "{}", error),
            Error::IoError(error) => write!(f, "{}", error),
            Error::WalkDirError(error) => write!(f, "{}", error),
            Error::StringError(error) => write!(f, "{}", error),
            Error::SledError(error) => write!(f, "{}", error),
            Error::SerdeCborError(error) => write!(f, "{}", error),
        }
    }
}

impl Into<tauri::InvokeError> for Error {
    fn into(self) -> tauri::InvokeError {
        tauri::InvokeError::from(Value::String(self.to_string()))
    }
}
