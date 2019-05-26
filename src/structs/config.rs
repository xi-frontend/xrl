use crate::ViewId;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigChanged {
    pub view_id: ViewId,
    pub changes: ConfigChanges
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigChanges {
    pub font_face: Option<String>,
    pub font_size: Option<u64>,
    pub line_ending: Option<String>,
    pub plugin_search_path: Option<Vec<String>>,
    pub tab_size: Option<u64>,
    pub translate_tabs_to_spaces: Option<bool>,
}
