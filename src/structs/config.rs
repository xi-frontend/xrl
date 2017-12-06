use ViewId;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigChanged {
    view_id: ViewId,
    changes: ConfigChanges
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigChanges {
    font_face: Option<String>,
    font_size: Option<u64>,
    line_ending: Option<String>,
    plugin_search_path: Option<Vec<String>>,
    tab_size: Option<u64>,
    translate_tabs_to_spaces: Option<bool>,
}
