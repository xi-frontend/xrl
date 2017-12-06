mod line;
mod operation;
mod style;
mod update;
mod position;
mod scroll_to;
mod plugins;
mod config;
mod view;

pub use self::line::{Line, StyleDef};
pub use self::operation::{Operation, OperationType};
pub use self::style::Style;
pub use self::update::Update;
pub use self::position::Position;
pub use self::scroll_to::ScrollTo;
pub use self::plugins::AvailablePlugins;
pub use self::plugins::Plugin;
pub use self::plugins::PluginStarted;
pub use self::plugins::PluginStoped;
pub use self::plugins::UpdateCmds;
pub use self::config::ConfigChanged;
pub use self::config::ConfigChanges;
pub use self::view::ViewId;

pub type ThemeSettings = ::syntect::highlighting::ThemeSettings;

#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeChanged {
    pub name: String,
    pub theme: ThemeSettings
}
