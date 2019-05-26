use crate::errors::ServerError;
use crate::protocol::Service;
use futures::{future, Future};
use serde_json::{from_value, Value};
use crate::structs::{
    AvailablePlugins, PluginStarted, PluginStoped,
    Update, ScrollTo, UpdateCmds, Style, ThemeChanged,
    ConfigChanged, Alert, AvailableThemes, FindStatus,
    ReplaceStatus, MeasureWidth, AvailableLanguages,
    LanguageChanged
};
use crate::client::Client;

pub type ServerResult<T> = Box<Future<Item = T, Error = ServerError>>;

/// Represents all possible RPC messages recieved from xi-core.
#[derive(Debug)]
pub enum XiEvent {
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
    MeasureWidth(MeasureWidth),
    AvailableLanguages(AvailableLanguages),
    LanguageChanged(LanguageChanged),
}

/// The `Frontend` trait must be implemented by clients. It defines how the
/// client handles notifications and requests coming from `xi-core`.
pub trait Frontend {

    fn handle_event(&mut self, e: XiEvent) -> ServerResult<()>;
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
                Ok(update) => self.handle_event(XiEvent::Update(update)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },

            "scroll_to" => match from_value::<ScrollTo>(params) {
                Ok(scroll_to) => self.handle_event(XiEvent::ScrollTo(scroll_to)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },

            "def_style" => match from_value::<Style>(params) {
                Ok(style) => self.handle_event(XiEvent::DefStyle(style)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "available_plugins" => match from_value::<AvailablePlugins>(params) {
                Ok(plugins) => self.handle_event(XiEvent::AvailablePlugins(plugins)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "plugin_started" => match from_value::<PluginStarted>(params) {
                Ok(plugin) => self.handle_event(XiEvent::PluginStarted(plugin)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "plugin_stoped" => match from_value::<PluginStoped>(params) {
                Ok(plugin) => self.handle_event(XiEvent::PluginStoped(plugin)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "update_cmds" => match from_value::<UpdateCmds>(params) {
                Ok(cmds) => self.handle_event(XiEvent::UpdateCmds(cmds)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "config_changed" => match from_value::<ConfigChanged>(params) {
                Ok(config) => self.handle_event(XiEvent::ConfigChanged(config)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "theme_changed" => match from_value::<ThemeChanged>(params) {
                Ok(theme) => self.handle_event(XiEvent::ThemeChanged(theme)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            },
            "alert" => match from_value::<Alert>(params) {
                Ok(alert) => self.handle_event(XiEvent::Alert(alert)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            }
            "available_themes" => match from_value::<AvailableThemes>(params) {
                Ok(themes) => self.handle_event(XiEvent::AvailableThemes(themes)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            }
            "find_status" => match from_value::<FindStatus>(params) {
                Ok(find_status) => self.handle_event(XiEvent::FindStatus(find_status)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            }
            "replace_status" => match from_value::<ReplaceStatus>(params) {
                Ok(replace_status) => self.handle_event(XiEvent::ReplaceStatus(replace_status)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            }
            "measure_width" => match from_value::<MeasureWidth>(params) {
                Ok(measure_width) => self.handle_event(XiEvent::MeasureWidth(measure_width)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            }
            "available_languages" => match from_value::<AvailableLanguages>(params) {
                Ok(available_langs) => self.handle_event(XiEvent::AvailableLanguages(available_langs)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            }
            "language_changed" => match from_value::<LanguageChanged>(params) {
                Ok(lang) => self.handle_event(XiEvent::LanguageChanged(lang)),
                Err(e) => Box::new(future::err(ServerError::DeserializeFailed(e))),
            }
            _ => Box::new(future::err(ServerError::UnknownMethod(method.into()))),
        }
    }
}
