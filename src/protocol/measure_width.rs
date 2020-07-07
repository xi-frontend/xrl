use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MeasureWidth(pub Vec<MeasureWidthInner>);

#[derive(Debug, Serialize, Deserialize)]
pub struct MeasureWidthInner {
    pub id: u64,
    pub strings: Vec<String>,
}
