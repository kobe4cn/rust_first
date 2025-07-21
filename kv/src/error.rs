use thiserror::Error;

#[derive(Error, Debug)]
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
    #[error("sled error: {0}")]
    SledError(#[from] sled::Error),
    #[error("prost error: {0}")]
    ProstError(#[from] prost::DecodeError),
    #[error("prost encode error: {0}")]
    ProstEncodeError(#[from] prost::EncodeError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("frame error")]
    FrameError,
    #[error("CertifcateParse error: {0} {1}")]
    CertParseError(String, String),
    #[error("rustls error: {0}")]
    RustlsError(#[from] tokio_rustls::rustls::Error),
}

impl PartialEq for KvError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (KvError::NotFound(t1, k1), KvError::NotFound(t2, k2)) => t1 == t2 && k1 == k2,
            (KvError::InvalidCommand(s1), KvError::InvalidCommand(s2)) => s1 == s2,
            (KvError::ConvertError(s1, t1), KvError::ConvertError(s2, t2)) => s1 == s2 && t1 == t2,
            (KvError::StorageError(c1, t1, k1, e1), KvError::StorageError(c2, t2, k2, e2)) => {
                c1 == c2 && t1 == t2 && k1 == k2 && e1 == e2
            }
            (KvError::Internal(s1), KvError::Internal(s2)) => s1 == s2,
            (KvError::KeyNotFound, KvError::KeyNotFound) => true,
            (KvError::SledError(_), KvError::SledError(_)) => false, // 无法比较 sled::Error
            (KvError::ProstError(_), KvError::ProstError(_)) => false, // 无法比较 prost::DecodeError
            (KvError::ProstEncodeError(_), KvError::ProstEncodeError(_)) => false, // 无法比较 prost::EncodeError
            (KvError::IoError(_), KvError::IoError(_)) => false, // 无法比较 std::io::Error
            (KvError::FrameError, KvError::FrameError) => true,
            _ => false,
        }
    }
}
