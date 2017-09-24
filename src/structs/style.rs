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
#[derive(Deserialize, Debug, PartialEq)]
pub struct Style {
    id: u64,
    fg_color: Option<u32>,
    #[serde(default = "default_bg_color")] bg_color: u32,
    #[serde(default = "default_weight")] weight: u32,
    #[serde(default = "default_italic")] italic: bool,
    #[serde(default = "default_underline")] underline: bool,
}
