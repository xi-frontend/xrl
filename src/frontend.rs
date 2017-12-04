use errors::ServerError;
use protocol::Service;
use futures::{future, Future};
use serde_json::{from_value, Value};
use structs::{
    AvailablePlugins, PluginStarted, PluginStoped,
    Update, ScrollTo, UpdateCmds, Style, ThemeChanged,
    ConfigChanged

};
use client::Client;

pub type ServerResult<T> = Box<Future<Item = T, Error = ServerError>>;

/// The `Frontend` trait must be implemented by clients. It defines how the
/// client handles notifications and requests coming from `xi-core`.
pub trait Frontend {
    /// handle `"updates"` notifications from `xi-core`
    fn update(&mut self, update: Update) -> ServerResult<()>;
    /// handle `"scroll_to"` notifications from `xi-core`
    fn scroll_to(&mut self, scroll_to: ScrollTo) -> ServerResult<()>;
    /// handle `"def_style"` notifications from `xi-core`
    fn def_style(&mut self, style: Style) -> ServerResult<()>;
    /// handle `"available_plugins"` notifications from `xi-core`
    fn available_plugins(&mut self, plugins: AvailablePlugins) -> ServerResult<()>;
    /// handle `"update_cmds"` notifications from `xi-core`
    fn update_cmds(&mut self, plugins: UpdateCmds) -> ServerResult<()>;
    /// handle `"plugin_started"` notifications from `xi-core`
    fn plugin_started(&mut self, plugins: PluginStarted) -> ServerResult<()>;
    /// handle `"plugin_stoped"` notifications from `xi-core`
    fn plugin_stoped(&mut self, plugin: PluginStoped) -> ServerResult<()>;
    /// handle `"config_changed"` notifications from `xi-core`
    fn config_changed(&mut self, config: ConfigChanged) -> ServerResult<()>;
    /// handle `"theme_changed"` notifications from `xi-core`
    fn theme_changed(&mut self, theme: ThemeChanged) -> ServerResult<()>;
}

/// A builder for the type `F` that implement the `Frontend` trait.
pub trait FrontendBuilder<F>
where
    F: Frontend,
{
    fn build(self, client: Client) -> F;
}

impl<F: Frontend> Service for F {
    type T = Value;
    type E = Value;
    type Error = ServerError;

    fn handle_request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Box<Future<Item = Result<Self::T, Self::E>, Error = Self::Error>> {
        // AFAIK the core does not send any request to frontends yet
        // We should return an ServerError here
        info!("<<< request: method={}, params={}", method, &params);
        unimplemented!();
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Box<Future<Item = (), Error = Self::Error>> {
        info!("<<< notification: method={}, params={}", method, &params);
        match method {
            "update" => match from_value::<Update>(params) {
                Ok(update) => self.update(update),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },

            "scroll_to" => match from_value::<ScrollTo>(params) {
                Ok(scroll_to) => self.scroll_to(scroll_to),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },

            "def_style" => match from_value::<Style>(params) {
                Ok(style) => self.def_style(style),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "available_plugins" => match from_value::<AvailablePlugins>(params) {
                Ok(plugins) => self.available_plugins(plugins),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e)))
            },
            "plugin_started" => match from_value::<PluginStarted>(params) {
                Ok(plugin) => self.plugin_started(plugin),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e)))
            },
            "plugin_stoped" => match from_value::<PluginStoped>(params) {
                Ok(plugin) => self.plugin_stoped(plugin),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e)))
            },
            "update_cmds" => match from_value::<UpdateCmds>(params) {
                Ok(cmds) => self.update_cmds(cmds),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "config_changed" => match from_value::<ConfigChanged>(params) {
                Ok(config) => self.config_changed(config),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e)))
            },
            "theme_changed" => match from_value::<ThemeChanged>(params) {
                Ok(theme) => self.theme_changed(theme),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e)))
            },

            _ => Box::new(future::err(ServerError::UnknownMethod(method.into()))),
        }
    }
}
