use serde::{Deserialize, Deserializer};

fn _return_true() -> bool {
    true
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct StyleDef {
    offset: i64,
    length: u64,
    style_id: u64,
}

#[derive(Deserialize)]
struct StyleDefHelper(i64, u64, u64);

impl<'de> Deserialize<'de> for StyleDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|StyleDefHelper(offset, length, style_id)| {
            StyleDef {
                offset: offset,
                length: length,
                style_id: style_id,
            }
        })
    }
}

#[derive(Default, Deserialize, Debug, PartialEq, Clone)]
pub struct Line {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub cursor: Vec<u64>,
    #[serde(default)]
    pub styles: Vec<StyleDef>,
    #[serde(default = "_return_true")]
    #[serde(skip_deserializing)]
    pub is_valid: bool,
}
