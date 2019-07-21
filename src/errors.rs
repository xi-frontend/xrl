use serde_json::error::Error as SerdeError;
use serde_json::Value;
use std::error;
use std::fmt;
use std::io::Error as IoError;

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

    /// We failed to spawn xi-core, e.g. because it's not installed, the binary is faulty, etc.
    CoreSpawnFailed(IoError),
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
            ClientError::CoreSpawnFailed(ref s) => {
                write!(f, "Failed to spawn xi-core due to error: {}", s)
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
            ClientError::SerializeFailed(_) => "Failed to serialize message",
            ClientError::CoreSpawnFailed(_) => "Failed to spawn xi-core",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            ClientError::SerializeFailed(ref serde_error) => Some(serde_error),
            ClientError::CoreSpawnFailed(ref io_error) => Some(io_error),
            _ => None,
        }
    }
}

impl From<SerdeError> for ClientError {
    fn from(err: SerdeError) -> Self {
        ClientError::SerializeFailed(err)
    }
}

impl From<IoError> for ClientError {
    fn from(err: IoError) -> Self {
        ClientError::CoreSpawnFailed(err)
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
