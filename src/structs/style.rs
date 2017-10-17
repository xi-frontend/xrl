fn default_bg_color() -> u32 {
    0
}
fn default_weight() -> u32 {
    400
}
fn default_italic() -> bool {
    false
}
fn default_underline() -> bool {
    false
}
#[derive(Default, Deserialize, Debug, PartialEq, Clone)]
pub struct Style {
    pub id: u64,
    pub fg_color: Option<u32>,
    #[serde(default = "default_bg_color")]
    pub bg_color: u32,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default = "default_italic")]
    pub italic: bool,
    #[serde(default = "default_underline")]
    pub underline: bool,
}
