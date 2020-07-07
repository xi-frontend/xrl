use serde::{Deserialize, Serialize};

use crate::protocol::ViewId;

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ScrollTo {
    pub line: u64,
    #[serde(rename = "col")]
    pub column: u64,
    pub view_id: ViewId,
}
