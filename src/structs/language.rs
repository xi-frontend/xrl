use super::view::ViewId;

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableLanguages {
    pub languages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageChanged {
    pub view_id: ViewId,
    pub language_id: String,
}
