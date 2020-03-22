//! `xrl` is a Tokio based library to build clients for the Xi editor. The
//! challenge with Xi RPC is that endpoints are both client (sending
//! requests/notifications) and server (handling incoming
//! requests/notifications).
//!
//!
//! ```rust
//! extern crate futures;
//! extern crate tokio;
//! extern crate xrl;
//!
//! use futures::{future, Future, Stream};
//! use xrl::*;
//!
//! // Type that represent a `xi-core` peer. It implements `Frontend`,
//! // which means it can handle notifications and requests from
//! // `xi-core`.
//! #[allow(dead_code)]
//! struct MyFrontend {
//!     // This is not actually used in this example, but if we wanted to
//!     // our frontend could use a `Client` so that it could send
//!     // requests and notifications to `xi-core`, instead of just
//!     // handling incoming messages.
//!     client: Client,
//! }
//!
//! // Implement how our client handles notifications & requests from the core.
//! impl Frontend for MyFrontend {
//!     type NotificationResult = Result<(), ()>;
//!     fn handle_notification(&mut self, notification: XiNotification) -> Self::NotificationResult {
//!         use XiNotification::*;
//!         match notification {
//!             Update(update) => println!("received `update` from Xi core:\n{:?}", update),
//!             ScrollTo(scroll) => println!("received `scroll_to` from Xi core:\n{:?}", scroll),
//!             DefStyle(style) => println!("received `def_style` from Xi core:\n{:?}", style),
//!             AvailablePlugins(plugins) => {
//!                 println!("received `available_plugins` from Xi core:\n{:?}", plugins)
//!             }
//!             UpdateCmds(cmds) => println!("received `update_cmds` from Xi core:\n{:?}", cmds),
//!             PluginStarted(plugin) => {
//!                 println!("received `plugin_started` from Xi core:\n{:?}", plugin)
//!             }
//!             PluginStoped(plugin) => {
//!                 println!("received `plugin_stoped` from Xi core:\n{:?}", plugin)
//!             }
//!             ConfigChanged(config) => {
//!                 println!("received `config_changed` from Xi core:\n{:?}", config)
//!             }
//!             ThemeChanged(theme) => println!("received `theme_changed` from Xi core:\n{:?}", theme),
//!             Alert(alert) => println!("received `alert` from Xi core:\n{:?}", alert),
//!             AvailableThemes(themes) => {
//!                 println!("received `available_themes` from Xi core:\n{:?}", themes)
//!             }
//!             FindStatus(status) => println!("received `find_status` from Xi core:\n{:?}", status),
//!             ReplaceStatus(status) => {
//!                 println!("received `replace_status` from Xi core:\n{:?}", status)
//!             }
//!             AvailableLanguages(langs) => {
//!                 println!("received `available_languages` from Xi core:\n{:?}", langs)
//!             }
//!             LanguageChanged(lang) => {
//!                 println!("received `language_changed` from Xi core:\n{:?}", lang)
//!             }
//!         }
//!         Ok(())
//!     }
//!
//!     type MeasureWidthResult = Result<Vec<Vec<f32>>, ()>;
//!     // we don't actually use the `request` argument in this example,
//!     // hence the attribute.
//!     #[allow(unused_variables)]
//!     fn handle_measure_width(&mut self, request: MeasureWidth) -> Self::MeasureWidthResult {
//!         Ok(Vec::new())
//!     }
//! }
//!
//! struct MyFrontendBuilder;
//!
//! impl FrontendBuilder for MyFrontendBuilder {
//!     type Frontend = MyFrontend;
//!     fn build(self, client: Client) -> Self::Frontend {
//!         MyFrontend { client }
//!     }
//! }
//!
//! fn init_xrl() {
//!     tokio::run(future::lazy(move || {
//!         // spawn Xi core
//!         let (client, core_stderr) = spawn("xi-core", MyFrontendBuilder {}).unwrap();
//!
//!         // start logging Xi core's stderr
//!         tokio::spawn(
//!             core_stderr
//!                 .for_each(|msg| {
//!                     println!("xi-core stderr: {}", msg);
//!                     Ok(())
//!                 })
//!                 .map_err(|_| ()),
//!         );
//!
//!         let client_clone = client.clone();
//!         client
//!             // Xi core expects the first notification to be
//!             // "client_started"
//!             .client_started(None, None)
//!             .map_err(|e| eprintln!("failed to send \"client_started\": {:?}", e))
//!             .and_then(move |_| {
//!                 let client = client_clone.clone();
//!                 client
//!                     .new_view(None)
//!                     .map(|view_name| println!("opened new view: {}", view_name))
//!                     .map_err(|e| eprintln!("failed to open a new view: {:?}", e))
//!                     .and_then(move |_| {
//!                         // Forces to shut down the Xi-RPC
//!                         // endoint. Otherwise, this example would keep
//!                         // running until the xi-core process
//!                         // terminates.
//!                         println!("shutting down");
//!                         client_clone.shutdown();
//!                         Ok(())
//!                     })
//!             })
//!     }));
//! }
//! ```

#![deny(clippy::all)]
#![allow(clippy::type_complexity)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

mod cache;
mod client;
mod core;
mod errors;
mod frontend;
mod protocol;
mod structs;

pub use crate::cache::LineCache;
pub use crate::client::Client;
pub use crate::core::{spawn, spawn_command, CoreStderr};
pub use crate::errors::{ClientError, ServerError};
pub use crate::frontend::{Frontend, FrontendBuilder, XiNotification};
pub use crate::protocol::IntoStaticFuture;
pub use crate::structs::{
    Alert, AvailableLanguages, AvailablePlugins, AvailableThemes, ConfigChanged, ConfigChanges,
    FindStatus, LanguageChanged, Line, MeasureWidth, ModifySelection, Operation, OperationType,
    PluginStarted, PluginStoped, Position, Query, ReplaceStatus, ScrollTo, Status, Style, StyleDef,
    ThemeChanged, ThemeSettings, Update, UpdateCmds, ViewId,
};
