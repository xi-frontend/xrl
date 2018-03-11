use std::io::Read;
use serde_json::{from_reader, to_vec, Value};

use super::errors::*;

#[derive(PartialEq, Clone, Debug)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

#[derive(Serialize, PartialEq, Clone, Debug)]
pub struct Request {
    pub id: u64,
    pub method: String,
    pub params: Value,
}

#[derive(Serialize, PartialEq, Clone, Debug)]
pub struct Response {
    pub id: u64,
    pub result: Result<Value, Value>,
}

#[derive(Serialize, PartialEq, Clone, Debug)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

impl Message {
    pub fn decode<R>(rd: &mut R) -> Result<Message, DecodeError>
    where
        R: Read,
    {
        let value = from_reader(rd)?;
        match get_message_type(&value) {
            ValueType::Request => Ok(Message::Request(Request::decode(value)?)),
            ValueType::Response => Ok(Message::Response(Response::decode(value)?)),
            ValueType::Notification => Ok(Message::Notification(Notification::decode(value)?)),
            ValueType::Invalid => Err(DecodeError::InvalidMessage),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        // According to serde_json's documentation for `to_value`:
        //
        // The conversion [of T to Value] can fail if T's implementation of
        // Serialize decides to
        // fail, or if T contains a map with non-string keys.
        //
        // This should not be the case here, so I think it's safe to unwrap.
        match *self {
            Message::Request(ref request) => to_vec(request).expect("Request serialization failed"),
            Message::Response(ref response) => {
                to_vec(response).expect("Response serialization failed")
            }
            Message::Notification(ref notification) => {
                to_vec(notification).expect("Notification serialization failed")
            }
        }
    }
}

impl Notification {
    fn decode(value: Value) -> Result<Self, DecodeError> {
        let mut value = value;
        let map = value.as_object_mut().ok_or(DecodeError::InvalidMessage)?;

        let method = map.remove("method")
            .ok_or(DecodeError::InvalidMessage)?
            .as_str()
            .ok_or(DecodeError::InvalidMessage)?
            .to_owned();

        let params = map.remove("params").ok_or(DecodeError::InvalidMessage)?;

        Ok(Notification {
            method,
            params,
        })
    }
}

impl Request {
    fn decode(value: Value) -> Result<Self, DecodeError> {
        let mut value = value;
        let map = value.as_object_mut().ok_or(DecodeError::InvalidMessage)?;

        let method = map.remove("method")
            .ok_or(DecodeError::InvalidMessage)?
            .as_str()
            .ok_or(DecodeError::InvalidMessage)?
            .to_owned();

        let params = map.remove("params").ok_or(DecodeError::InvalidMessage)?;

        let id = map.remove("id")
            .ok_or(DecodeError::InvalidMessage)?
            .as_u64()
            .ok_or(DecodeError::InvalidMessage)?;

        Ok(Request {
            id,
            method,
            params,
        })
    }
}

impl Response {
    fn decode(value: Value) -> Result<Self, DecodeError> {
        let mut value = value;
        let map = value.as_object_mut().ok_or(DecodeError::InvalidMessage)?;

        let result = if map.contains_key("result") {
            Ok(map.remove("result").ok_or(DecodeError::InvalidMessage)?)
        } else if map.contains_key("error") {
            Err(map.remove("error").ok_or(DecodeError::InvalidMessage)?)
        } else {
            return Err(DecodeError::InvalidMessage);
        };

        let id = map.remove("id")
            .ok_or(DecodeError::InvalidMessage)?
            .as_u64()
            .ok_or(DecodeError::InvalidMessage)?;

        Ok(Response {
            id,
            result,
        })
    }
}

enum ValueType {
    Request,
    Response,
    Notification,
    Invalid,
}

fn get_message_type(value: &Value) -> ValueType {
    if let Value::Object(ref map) = *value {
        if map.contains_key("method") && map.contains_key("params") {
            if map.contains_key("id") {
                ValueType::Request
            } else {
                ValueType::Notification
            }
        } else if (map.contains_key("result") || map.contains_key("error"))
            && map.contains_key("id")
        {
            ValueType::Response
        } else {
            ValueType::Invalid
        }
    } else {
        ValueType::Invalid
    }
}
