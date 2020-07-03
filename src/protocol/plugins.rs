use serde::{Deserialize, Serialize};

use crate::protocol::ViewId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AvailablePlugins {
    pub view_id: ViewId,
    pub plugins: Vec<Plugin>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginStarted {
    pub view_id: ViewId,
    pub plugin: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginStoped {
    pub view_id: ViewId,
    pub plugin: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateCmds {
    pub cmds: Vec<String>,
    pub plugin: String,
    pub view_id: ViewId,
}
