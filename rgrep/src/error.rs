use thiserror::Error;

#[derive(Debug, Error)]
pub enum GrepError {
    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("other error: {0}")]
    Other(String),
}
