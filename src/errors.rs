use std::fmt;
use std::error::{self, Error};
use serde_json::Value;
use serde_json::error::Error as SerdeError;

#[derive(Debug)]
pub enum ClientError {
    /// A notification was not sent due to an internal error.
    NotifyFailed,
    /// A request failed due to an internal error.
    RequestFailed,

    /// A request or a notification could not be sent due to a serialization error.
    SerializeFailed(SerdeError),

    /// The server response is an error
    ErrorReturned(Value),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClientError::NotifyFailed => write!(f, "Failed to send a notification"),
            ClientError::RequestFailed => {
                write!(f, "Failed to send a request, or receive its response")
            }
            ClientError::ErrorReturned(ref value) => {
                write!(f, "The core returned an error: {:?}", value)
            }
            ClientError::SerializeFailed(ref e) => {
                write!(f, "failed to serialize a message: {}", e)
            }
        }
    }
}

impl error::Error for ClientError {
    fn description(&self) -> &str {
        match *self {
            ClientError::NotifyFailed => "Failed to send a notification",
            ClientError::RequestFailed => "Failed to send a request or receive its response",
            ClientError::ErrorReturned(_) => "The core answered with an error",
            ClientError::SerializeFailed(_) => "failed to serialize message",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        if let ClientError::SerializeFailed(ref serde_error) = *self {
            Some(serde_error)
        } else {
            None
        }
    }
}

impl From<SerdeError> for ClientError {
    fn from(err: SerdeError) -> Self {
        ClientError::SerializeFailed(err)
    }
}
