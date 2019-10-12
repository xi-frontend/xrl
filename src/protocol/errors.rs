use serde_json::error::Category;
use serde_json::error::Error as SerdeError;
use std::io;

#[derive(Debug)]
pub enum DecodeError {
    Truncated,
    Io(io::Error),
    InvalidJson,
}

impl From<SerdeError> for DecodeError {
    fn from(err: SerdeError) -> DecodeError {
        match err.classify() {
            Category::Io => DecodeError::Io(err.into()),
            Category::Eof => DecodeError::Truncated,
            Category::Data | Category::Syntax => DecodeError::InvalidJson,
        }
    }
}

#[derive(Debug)]
pub enum RpcError {
    ResponseCanceled,
    AckCanceled,
}
