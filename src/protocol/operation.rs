use serde::{Deserialize, Serialize};

use super::line::Line;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OperationType {
    Copy,
    Skip,
    Invalidate,
    Update,
    #[serde(rename = "ins")]
    Insert,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Operation {
    #[serde(rename = "op")]
    pub operation_type: OperationType,
    #[serde(rename = "n")]
    pub nb_lines: u64,
    #[serde(rename = "ln")]
    pub line_num: Option<u64>,
    #[serde(default)]
    pub lines: Vec<Line>,
}
