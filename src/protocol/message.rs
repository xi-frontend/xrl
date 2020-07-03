use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::protocol::XiNotification;

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Error(String),
    Request(Request),
    Response(Response),
    Notification(XiNotification),
}

#[derive(Serialize, Clone, Debug, PartialEq, Deserialize)]
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

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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
