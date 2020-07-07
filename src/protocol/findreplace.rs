use serde::{Deserialize, Serialize};

use crate::protocol::ViewId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Query {
    pub id: u64,
    pub chars: Option<String>,
    pub case_sensitive: Option<bool>,
    pub is_regex: Option<bool>,
    pub whole_words: Option<bool>,
    pub matches: u64,
    pub lines: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FindStatus {
    pub view_id: ViewId,
    pub queries: Vec<Query>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Status {
    pub chars: String,
    pub preserve_case: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplaceStatus {
    pub view_id: ViewId,
    pub status: Status,
}
