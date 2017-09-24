use serde;
use serde_json as json;

fn _return_true() -> bool {
    true
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Line {
    #[serde(default)] pub text: String,
    #[serde(default)]
    #[serde(rename = "cursor")]
    pub cursors: Vec<u64>,
    #[serde(default)] pub styles: Vec<i64>,
    #[serde(default = "_return_true")]
    #[serde(skip_deserializing)]
    pub is_valid: bool,
}
