use crate::client::Client;
use crate::protocol::Service;
use crate::structs::{
    Alert, AvailableLanguages, AvailablePlugins, AvailableThemes, ConfigChanged, FindStatus,
    LanguageChanged, MeasureWidth, PluginStarted, PluginStoped, ReplaceStatus, ScrollTo, Style,
    ThemeChanged, Update, UpdateCmds,
};
use futures::{future, Future};
use serde_json::{from_value, to_value, Value};

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

pub type ServerResult<T, E> = Box<dyn Future<Item = T, Error = E> + Send + 'static>;

/// The `Frontend` trait must be implemented by clients. It defines how the
/// client handles notifications and requests coming from `xi-core`.
pub trait Frontend {
    fn handle_notification(&mut self, notification: XiNotification) -> ServerResult<(), ()>;
    fn handle_measure_width(&mut self, request: MeasureWidth) -> ServerResult<Vec<Vec<f32>>, ()>;
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
    type RequestFuture = Box<dyn Future<Item = Self::T, Error = Self::E> + 'static + Send>;
    type NotificationFuture = Box<dyn Future<Item = (), Error = ()> + Send + 'static>;

    fn handle_request(&mut self, method: &str, params: Value) -> Self::RequestFuture {
        info!("<<< request: method={}, params={}", method, &params);
        match method {
            "measure_width" => {
                match from_value::<MeasureWidth>(params) {
                    Ok(request) => {
                        let future = self
                            .handle_measure_width(request)
                            .map(|response| {
                                // TODO: justify why this can't fail
                                // https://docs.serde.rs/serde_json/value/fn.to_value.html#errors
                                to_value(response).expect("failed to convert response")
                            })
                            .map_err(|_| panic!("errors are not supported"));
                        Box::new(future)
                    }
                    Err(e) => {
                        warn!("failed to deserialize measure_width message: {:?}", e);
                        let err_msg = to_value("invalid measure_width message")
                            // TODO: justify why string serialization cannot fail
                            .expect("failed to serialize string");
                        Box::new(future::err(err_msg))
                    }
                }
            }
            _ => {
                let err_msg = to_value(format!("unknown method \"{}\"", method))
                    // TODO: justify why string serialization cannot fail
                    .expect("failed to serialize string");
                Box::new(future::err(err_msg))
            }
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn handle_notification(&mut self, method: &str, params: Value) -> Self::NotificationFuture {
        info!("<<< notification: method={}, params={}", method, &params);
        match method {
            "update" => match from_value::<Update>(params) {
                Ok(update) => self.handle_notification(XiNotification::Update(update)),
                Err(e) => {
                    error!("received invalid update notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },

            "scroll_to" => match from_value::<ScrollTo>(params) {
                Ok(scroll_to) => self.handle_notification(XiNotification::ScrollTo(scroll_to)),
                Err(e) => {
                    error!("received invalid scroll_to notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },

            "def_style" => match from_value::<Style>(params) {
                Ok(style) => self.handle_notification(XiNotification::DefStyle(style)),
                Err(e) => {
                    error!("received invalid def_style notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "available_plugins" => match from_value::<AvailablePlugins>(params) {
                Ok(plugins) => self.handle_notification(XiNotification::AvailablePlugins(plugins)),
                Err(e) => {
                    error!("received invalid available_plugins notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "plugin_started" => match from_value::<PluginStarted>(params) {
                Ok(plugin) => self.handle_notification(XiNotification::PluginStarted(plugin)),
                Err(e) => {
                    error!("received invalid plugin_started notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "plugin_stoped" => match from_value::<PluginStoped>(params) {
                Ok(plugin) => self.handle_notification(XiNotification::PluginStoped(plugin)),
                Err(e) => {
                    error!("received invalid plugin_stoped notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "update_cmds" => match from_value::<UpdateCmds>(params) {
                Ok(cmds) => self.handle_notification(XiNotification::UpdateCmds(cmds)),
                Err(e) => {
                    error!("received invalid update_cmds notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "config_changed" => match from_value::<ConfigChanged>(params) {
                Ok(config) => self.handle_notification(XiNotification::ConfigChanged(config)),
                Err(e) => {
                    error!("received invalid config_changed notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "theme_changed" => match from_value::<ThemeChanged>(params) {
                Ok(theme) => self.handle_notification(XiNotification::ThemeChanged(theme)),
                Err(e) => {
                    error!("received invalid theme_changed notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "alert" => match from_value::<Alert>(params) {
                Ok(alert) => self.handle_notification(XiNotification::Alert(alert)),
                Err(e) => {
                    error!("received invalid alert notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "available_themes" => match from_value::<AvailableThemes>(params) {
                Ok(themes) => self.handle_notification(XiNotification::AvailableThemes(themes)),
                Err(e) => {
                    error!("received invalid available_themes notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "find_status" => match from_value::<FindStatus>(params) {
                Ok(find_status) => {
                    self.handle_notification(XiNotification::FindStatus(find_status))
                }
                Err(e) => {
                    error!("received invalid find_status notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "replace_status" => match from_value::<ReplaceStatus>(params) {
                Ok(replace_status) => {
                    self.handle_notification(XiNotification::ReplaceStatus(replace_status))
                }
                Err(e) => {
                    error!("received invalid replace_status notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "available_languages" => match from_value::<AvailableLanguages>(params) {
                Ok(available_langs) => {
                    self.handle_notification(XiNotification::AvailableLanguages(available_langs))
                }
                Err(e) => {
                    error!("received invalid available_languages notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            "language_changed" => match from_value::<LanguageChanged>(params) {
                Ok(lang) => self.handle_notification(XiNotification::LanguageChanged(lang)),
                Err(e) => {
                    error!("received invalid language_changed notification: {:?}", e);
                    Box::new(future::err(()))
                }
            },
            _ => Box::new(future::err(())),
        }
    }
}
