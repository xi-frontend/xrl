use serde::{Deserialize, Deserializer, Serializer};
use serde_json::{from_reader, to_vec, Value};
use std::io::Read;

use super::errors::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: String,
    pub params: Value,
}

fn serialize_json_rpc_result<S>(
    val: &Result<Value, Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match val {
        Ok(v) => serializer.serialize_newtype_variant("", 0, "result", v),
        Err(v) => serializer.serialize_newtype_variant("", 1, "error", v),
    }
}

pub fn deserialize_json_rpc_result<'de, D>(
    deserializer: D,
) -> Result<Result<Value, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    match JsonRpcResult::<Value, Value>::deserialize(deserializer)? {
        JsonRpcResult::Result(value) => Ok(Ok(value)),
        JsonRpcResult::Error(value) => Ok(Err(value)),
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Response {
    pub id: u64,
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_json_rpc_result")]
    #[serde(deserialize_with = "deserialize_json_rpc_result")]
    pub result: Result<Value, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum JsonRpcResult<T, E> {
    Result(T),
    Error(E),
}

#[derive(Serialize, PartialEq, Clone, Debug, Deserialize)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

impl Message {
    pub fn decode<R>(rd: &mut R) -> Result<Message, DecodeError>
    where
        R: Read,
    {
        Ok(from_reader(rd)?)
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

#[test]
fn test_decode_message_ok() {
    let s = r#"{"id": 1, "result": "foo"}"#;
    let expected = Response {
        id: 1,
        result: Ok(Value::String(String::from("foo"))),
    };
    let actual: Response = serde_json::from_str(s).unwrap();
    assert_eq!(actual.id, expected.id);
    assert_eq!(actual.result, expected.result);
}

#[test]
fn test_decode_message_err() {
    let s = r#"{"id": 1, "error": "foo"}"#;
    let expected = Response {
        id: 1,
        result: Err(Value::String(String::from("foo"))),
    };
    let actual: Response = serde_json::from_str(s).unwrap();
    assert_eq!(actual.id, expected.id);
    assert_eq!(actual.result, expected.result);
}
