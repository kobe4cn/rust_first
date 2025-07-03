use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum KvError {
    #[error("Not found for table: {0}, key: {1}")]
    NotFound(String, String),
    #[error("Command parse error: {0}")]
    InvalidCommand(String),
    #[error("Convert value error: {0} to {1}")]
    ConvertError(String, &'static str),
    #[error("storage error: with command: {0}, table: {1}, key: {2}, error: {3}")]
    StorageError(&'static str, String, String, String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("key not found")]
    KeyNotFound,
}
