use ViewId;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Plugin {
    name: String,
    running: bool
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AvailablePlugins {
    pub view_id: ViewId,
    pub plugins: Vec<Plugin>
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginStarted {
    pub view_id: ViewId,
    pub plugin: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginStoped {
    pub view_id: ViewId,
    pub plugin: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateCmds {
    cmds: Vec<String>,
    plugin: String,
    view_id: ViewId
}
