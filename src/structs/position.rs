use serde::{Serialize, Serializer, Deserialize, Deserializer};

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Position(pub u64, pub u64);

#[test]
fn deserialize_ok() {
    use serde_json;

    let s = r#"[12, 1]"#;
    let deserialized: Result<Position, _> = serde_json::from_str(s);
    assert_eq!(deserialized.unwrap(), Position(12, 1));
}
