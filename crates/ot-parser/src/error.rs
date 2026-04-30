use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Checksum mismatch: expected {expected:#06x}, got {actual:#06x}")]
    ChecksumMismatch { expected: u16, actual: u16 },

    #[error("Invalid magic bytes")]
    InvalidMagic,

    #[error("Unexpected file size: expected {expected}, got {actual}")]
    UnexpectedSize { expected: usize, actual: usize },
}

pub type Result<T> = std::result::Result<T, ParseError>;
