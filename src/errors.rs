use std::fmt;
use std::error::{self, Error};
use serde_json::Value;
use serde_json::error::Error as SerdeError;

#[derive(Debug)]
pub enum RpcError {
    /// Failure to send a notification
    NotificationNotSent,
    // FIXME: we should be able to provide a better error than this and know what went wrong, but
    // that needs to be fixed in the core
    /// Failure to send a request or to receive a response
    UnknownRequestError,
    /// Error while serializing or deserializing a message
    Serde(SerdeError),
    /// The server returned an error to a request
    RequestError(Value),
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RpcError::NotificationNotSent => write!(f, "Failed to send notification"),
            RpcError::UnknownRequestError => write!(
                f,
                "Failed to send a request, or receive a request's response"
            ),
            RpcError::RequestError(ref value) => {
                write!(f, "The core returned an error: {:?}", value)
            }
            RpcError::Serde(ref e) => {
                write!(f, "failed to (de)serialize a message: {}", e.description())
            }
        }
    }
}

impl error::Error for RpcError {
    fn description(&self) -> &str {
        match *self {
            RpcError::NotificationNotSent => "Failed to send notification",
            RpcError::UnknownRequestError => {
                "Failed to send a request or to receive a request's response"
            }
            RpcError::RequestError(_) => "The core answered with an error",
            RpcError::Serde(_) => "failed to serialize/deserialize a message",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        if let RpcError::Serde(ref serde_error) = *self {
            Some(serde_error)
        } else {
            None
        }
    }
}

impl From<SerdeError> for RpcError {
    fn from(err: SerdeError) -> Self {
        RpcError::Serde(err)
    }
}
