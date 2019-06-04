use crate::client::Client;
use crate::errors::ServerError;
use crate::protocol::Service;
use futures::{future, Future};
use serde_json::{from_value, to_value, Value};
use crate::structs::{
    AvailablePlugins, PluginStarted, PluginStoped,
    Update, ScrollTo, UpdateCmds, Style, ThemeChanged,
    ConfigChanged, Alert, AvailableThemes, FindStatus,
    ReplaceStatus, MeasureWidth, AvailableLanguages,
    LanguageChanged
};

pub type ServerResult<T> = Box<Future<Item = T, Error = ServerError>>;

/// Represents all possible RPC messages recieved from xi-core.
#[derive(Debug)]
pub enum XiNotification {
    Update(Update),
    ScrollTo(ScrollTo),
    DefStyle(Style),
    AvailablePlugins(AvailablePlugins),
    UpdateCmds(UpdateCmds),
    PluginStarted(PluginStarted),
    PluginStoped(PluginStoped),
    ConfigChanged(ConfigChanged),
    ThemeChanged(ThemeChanged),
    Alert(Alert),
    AvailableThemes(AvailableThemes),
    FindStatus(FindStatus),
    ReplaceStatus(ReplaceStatus),
    AvailableLanguages(AvailableLanguages),
    LanguageChanged(LanguageChanged),
}

/// The `Frontend` trait must be implemented by clients. It defines how the
/// client handles notifications and requests coming from `xi-core`.
pub trait Frontend {
    fn handle_notification(&mut self, notification: XiNotification) -> ServerResult<()>;
    fn handle_measure_width(&mut self, request: MeasureWidth) -> ServerResult<Vec<Vec<f32>>>;
}

/// A builder for the type `F` that implement the `Frontend` trait.
pub trait FrontendBuilder<F>
where
    F: Frontend,
{
    fn build(self, client: Client) -> F;
}

impl<F: Frontend + Send> Service for F {
    type T = Value;
    type E = Value;
    type Error = ServerError;

    fn handle_request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Box<Future<Item = Result<Self::T, Self::E>, Error = Self::Error>> {
        info!("<<< request: method={}, params={}", method, &params);
        match method {
            "measure_width" => match from_value::<MeasureWidth>(params) {
                Ok(req) => Box::new(
                    self.handle_measure_width(req)
                        .and_then(|resp| Ok(Ok(to_value(resp).map_err(ServerError::from)?))),
                ),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            _ => Box::new(future::err(ServerError::UnknownMethod(method.into()))),
        }
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Box<Future<Item = (), Error = Self::Error>> {
        info!("<<< notification: method={}, params={}", method, &params);
        match method {
            "update" => match from_value::<Update>(params) {
                Ok(update) => self.handle_notification(XiNotification::Update(update)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },

            "scroll_to" => match from_value::<ScrollTo>(params) {
                Ok(scroll_to) => self.handle_notification(XiNotification::ScrollTo(scroll_to)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },

            "def_style" => match from_value::<Style>(params) {
                Ok(style) => self.handle_notification(XiNotification::DefStyle(style)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "available_plugins" => match from_value::<AvailablePlugins>(params) {
                Ok(plugins) => self.handle_notification(XiNotification::AvailablePlugins(plugins)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "plugin_started" => match from_value::<PluginStarted>(params) {
                Ok(plugin) => self.handle_notification(XiNotification::PluginStarted(plugin)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "plugin_stoped" => match from_value::<PluginStoped>(params) {
                Ok(plugin) => self.handle_notification(XiNotification::PluginStoped(plugin)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "update_cmds" => match from_value::<UpdateCmds>(params) {
                Ok(cmds) => self.handle_notification(XiNotification::UpdateCmds(cmds)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "config_changed" => match from_value::<ConfigChanged>(params) {
                Ok(config) => self.handle_notification(XiNotification::ConfigChanged(config)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "theme_changed" => match from_value::<ThemeChanged>(params) {
                Ok(theme) => self.handle_notification(XiNotification::ThemeChanged(theme)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "alert" => match from_value::<Alert>(params) {
                Ok(alert) => self.handle_notification(XiNotification::Alert(alert)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "available_themes" => match from_value::<AvailableThemes>(params) {
                Ok(themes) => self.handle_notification(XiNotification::AvailableThemes(themes)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "find_status" => match from_value::<FindStatus>(params) {
                Ok(find_status) => {
                    self.handle_notification(XiNotification::FindStatus(find_status))
                }
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "replace_status" => match from_value::<ReplaceStatus>(params) {
                Ok(replace_status) => {
                    self.handle_notification(XiNotification::ReplaceStatus(replace_status))
                }
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "available_languages" => match from_value::<AvailableLanguages>(params) {
                Ok(available_langs) => {
                    self.handle_notification(XiNotification::AvailableLanguages(available_langs))
                }
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "language_changed" => match from_value::<LanguageChanged>(params) {
                Ok(lang) => self.handle_notification(XiNotification::LanguageChanged(lang)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            _ => Box::new(future::err(ServerError::UnknownMethod(method.into()))),
        }
    }
}
