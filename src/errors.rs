use std::fmt;
use std::error;
use serde_json::Value;
use serde_json::error::Error as SerdeError;

#[derive(Debug)]
pub enum ClientError {
    /// A notification was not sent due to an internal error.
    NotifyFailed,
    /// A request failed due to an internal error.
    RequestFailed,

    /// A request or a notification could not be sent due to a
    /// serialization error.
    SerializeFailed(SerdeError),

    /// The server response is an error
    ErrorReturned(Value),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    fn cause(&self) -> Option<&dyn error::Error> {
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

#[derive(Debug)]
pub enum ServerError {
    UnknownMethod(String),
    DeserializeFailed(SerdeError),
    Other(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ServerError::UnknownMethod(ref method) => write!(f, "Unkown method {}", method),
            ServerError::Other(ref s) => write!(f, "Unkown error: {}", s),
            ServerError::DeserializeFailed(ref e) => write!(
                f,
                "Failed to deserialize the parameters of a request or notification: {}",
                e
            ),
        }
    }
}

impl error::Error for ServerError {
    fn description(&self) -> &str {
        match *self {
            ServerError::UnknownMethod(_) => "Unkown method",
            ServerError::Other(_) => "Unknown error",
            ServerError::DeserializeFailed(_) => {
                "Failed to deserialize the parameters of a request or notification"
            }
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        if let ServerError::DeserializeFailed(ref serde_error) = *self {
            Some(serde_error)
        } else {
            None
        }
    }
}

impl From<String> for ServerError {
    fn from(s: String) -> Self {
        ServerError::Other(s)
    }
}

impl<'a> From<&'a str> for ServerError {
    fn from(s: &'a str) -> Self {
        ServerError::Other(s.to_string())
    }
}

impl From<SerdeError> for ServerError {
    fn from(err: SerdeError) -> Self {
        ServerError::DeserializeFailed(err)
    }
}
