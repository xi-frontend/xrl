use serde;
use serde_json as json;

use super::line::Line;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub enum OperationType {
    Copy_,
    Skip,
    Invalidate,
    Update,
    Insert,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Operation {
    #[serde(rename = "op")]
    #[serde(deserialize_with = "deserialize_operation_type")]
    pub operation_type: OperationType,
    #[serde(rename = "n")]
    pub nb_lines: u64,
    #[serde(default)]
    pub lines: Vec<Line>,
}

fn deserialize_operation_type<'de, D>(de: D) -> ::std::result::Result<OperationType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: json::Value = try!(serde::Deserialize::deserialize(de));
    match value {
        json::Value::String(ref s) if &*s == "copy" => Ok(OperationType::Copy_),
        json::Value::String(ref s) if &*s == "skip" => Ok(OperationType::Skip),
        json::Value::String(ref s) if &*s == "invalidate" => Ok(OperationType::Invalidate),
        json::Value::String(ref s) if &*s == "update" => Ok(OperationType::Update),
        json::Value::String(ref s) if &*s == "ins" => Ok(OperationType::Insert),
        _ => Err(serde::de::Error::custom(
            "Unexpected value for operation type",
        )),
    }
}

#[test]
fn deserialize_operation_from_value() {
    use serde_json;

    let value = json!({"n": 12, "op": "ins"});
    let operation = Operation {
        operation_type: OperationType::Insert,
        nb_lines: 12,
        lines: vec![],
    };
    let deserialized: Result<Operation, _> = serde_json::from_value(value);
    assert_eq!(deserialized.unwrap(), operation);

    let value =
        json!({"lines":[{"cursor":[0],"styles":[],"text":"foo"},{"styles":[],"text":""}],"n":60,"op":"invalidate"});
    let operation = Operation {
        operation_type: OperationType::Invalidate,
        nb_lines: 60,
        lines: vec![
            Line {
                cursor: vec![0],
                styles: vec![],
                text: "foo".to_owned(),
            },
            Line {
                cursor: vec![],
                styles: vec![],
                text: "".to_owned(),
            },
        ],
    };
    let deserialized: Result<Operation, _> = serde_json::from_value(value);
    assert_eq!(deserialized.unwrap(), operation);
}

#[test]
fn deserialize_operation() {
    use serde_json;

    let s = r#"{"n": 12, "op": "ins"}"#;
    let operation = Operation {
        operation_type: OperationType::Insert,
        nb_lines: 12,
        lines: vec![],
    };
    let deserialized: Result<Operation, _> = serde_json::from_str(s);
    assert_eq!(deserialized.unwrap(), operation);


    let s =
        r#"{"lines":[{"cursor":[0],"styles":[],"text":"foo"},{"styles":[],"text":""}],"n":60,"op":"invalidate"}"#;
    let operation = Operation {
        operation_type: OperationType::Invalidate,
        nb_lines: 60,
        lines: vec![
            Line {
                cursor: vec![0],
                styles: vec![],
                text: "foo".to_owned(),
            },
            Line {
                cursor: vec![],
                styles: vec![],
                text: "".to_owned(),
            },
        ],
    };
    let deserialized: Result<Operation, _> = serde_json::from_str(s);
    assert_eq!(deserialized.unwrap(), operation);
}
