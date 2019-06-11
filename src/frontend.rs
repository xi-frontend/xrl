use crate::client::Client;
use crate::protocol::{Client as InnerClient, IntoStaticFuture, Service, ServiceBuilder};
use crate::structs::{
    Alert, AvailableLanguages, AvailablePlugins, AvailableThemes, ConfigChanged, FindStatus,
    LanguageChanged, MeasureWidth, PluginStarted, PluginStoped, ReplaceStatus, ScrollTo, Style,
    ThemeChanged, Update, UpdateCmds,
};
use futures::{
    future::{self, Either, FutureResult},
    Future,
};
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

/// The `Frontend` trait must be implemented by clients. It defines how the
/// client handles notifications and requests coming from `xi-core`.
pub trait Frontend {
    type NotificationResult: IntoStaticFuture<Item = (), Error = ()>;
    fn handle_notification(&mut self, notification: XiNotification) -> Self::NotificationResult;

    type MeasureWidthResult: IntoStaticFuture<Item = Vec<Vec<f32>>, Error = ()>;
    fn handle_measure_width(&mut self, request: MeasureWidth) -> Self::MeasureWidthResult;
}

/// A trait to build a type that implements `Frontend`.
pub trait FrontendBuilder {
    /// The type to build
    type Frontend: Frontend;

    /// Build the frontend with the given client.
    fn build(self, client: Client) -> Self::Frontend;
}

impl<B> ServiceBuilder for B
where
    B: FrontendBuilder,
    B::Frontend: Send,
{
    type Service = B::Frontend;

    fn build(self, client: InnerClient) -> B::Frontend {
        <Self as FrontendBuilder>::build(self, Client(client))
    }
}

impl<F: Frontend + Send> Service for F {
    type T = Value;
    type E = Value;
    type RequestFuture = Box<dyn Future<Item = Self::T, Error = Self::E> + 'static + Send>;
    type NotificationFuture = Either<
        <<F as Frontend>::NotificationResult as IntoStaticFuture>::Future,
        FutureResult<(), ()>,
    >;

    fn handle_request(&mut self, method: &str, params: Value) -> Self::RequestFuture {
        info!("<<< request: method={}, params={}", method, &params);
        match method {
            "measure_width" => {
                match from_value::<MeasureWidth>(params) {
                    Ok(request) => {
                        let future = self
                            .handle_measure_width(request)
                            .into_static_future()
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
                Ok(update) => Either::A(
                    self.handle_notification(XiNotification::Update(update))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid update notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },

            "scroll_to" => match from_value::<ScrollTo>(params) {
                Ok(scroll_to) => Either::A(
                    self.handle_notification(XiNotification::ScrollTo(scroll_to))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid scroll_to notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },

            "def_style" => match from_value::<Style>(params) {
                Ok(style) => Either::A(
                    self.handle_notification(XiNotification::DefStyle(style))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid def_style notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "available_plugins" => match from_value::<AvailablePlugins>(params) {
                Ok(plugins) => Either::A(
                    self.handle_notification(XiNotification::AvailablePlugins(plugins))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid available_plugins notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "plugin_started" => match from_value::<PluginStarted>(params) {
                Ok(plugin) => Either::A(
                    self.handle_notification(XiNotification::PluginStarted(plugin))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid plugin_started notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "plugin_stoped" => match from_value::<PluginStoped>(params) {
                Ok(plugin) => Either::A(
                    self.handle_notification(XiNotification::PluginStoped(plugin))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid plugin_stoped notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "update_cmds" => match from_value::<UpdateCmds>(params) {
                Ok(cmds) => Either::A(
                    self.handle_notification(XiNotification::UpdateCmds(cmds))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid update_cmds notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "config_changed" => match from_value::<ConfigChanged>(params) {
                Ok(config) => Either::A(
                    self.handle_notification(XiNotification::ConfigChanged(config))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid config_changed notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "theme_changed" => match from_value::<ThemeChanged>(params) {
                Ok(theme) => Either::A(
                    self.handle_notification(XiNotification::ThemeChanged(theme))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid theme_changed notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "alert" => match from_value::<Alert>(params) {
                Ok(alert) => Either::A(
                    self.handle_notification(XiNotification::Alert(alert))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid alert notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "available_themes" => match from_value::<AvailableThemes>(params) {
                Ok(themes) => Either::A(
                    self.handle_notification(XiNotification::AvailableThemes(themes))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid available_themes notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "find_status" => match from_value::<FindStatus>(params) {
                Ok(find_status) => Either::A(
                    self.handle_notification(XiNotification::FindStatus(find_status))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid find_status notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "replace_status" => match from_value::<ReplaceStatus>(params) {
                Ok(replace_status) => Either::A(
                    self.handle_notification(XiNotification::ReplaceStatus(replace_status))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid replace_status notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "available_languages" => match from_value::<AvailableLanguages>(params) {
                Ok(available_langs) => Either::A(
                    self.handle_notification(XiNotification::AvailableLanguages(available_langs))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid available_languages notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            "language_changed" => match from_value::<LanguageChanged>(params) {
                Ok(lang) => Either::A(
                    self.handle_notification(XiNotification::LanguageChanged(lang))
                        .into_static_future(),
                ),
                Err(e) => {
                    error!("received invalid language_changed notification: {:?}", e);
                    Either::B(future::err(()))
                }
            },
            _ => Either::B(future::err(())),
        }
    }
}
