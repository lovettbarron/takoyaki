use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Database error: {0}")]
    Db(String),

    #[error("Device error: {0}")]
    Device(String),

    #[error("Invalid path")]
    InvalidPath,
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Db(e.to_string())
    }
}

impl From<ot_parser::ParseError> for AppError {
    fn from(e: ot_parser::ParseError) -> Self {
        AppError::Parse(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
