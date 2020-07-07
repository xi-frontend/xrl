use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModifySelection {
    None,
    Set,
    Add,
    AddRemoveCurrent,
}
