//! `xrl` is a Tokio based library to build clients for the Xi editor. The
//! challenge with Xi RPC is that endpoints are both client (sending
//! requests/notifications) and server (handling incoming
//! requests/notifications).
//!
//!
//! ```rust
//!
//! #![allow(unused_variables)]
//! extern crate futures;
//! extern crate tokio;
//! extern crate xrl;
//!
//! use futures::{future, Future, Stream};
//! use xrl::*;
//!
//!
//! // Type that represent our client
//! struct MyFrontend {
//!     client: Client,
//! }
//!
//! // Implement how our client handles notifications & requests from the core.
//! impl Frontend for MyFrontend {
//!
//!     fn handle_notification(&mut self, notification: XiNotification) -> ServerResult<()> {
//!         use XiNotification::*;
//!         match notification {
//!             Update(update) => println!("received `update` from Xi core:\n{:?}", update),
//!             ScrollTo(scroll) => println!("received `scroll_to` from Xi core:\n{:?}", scroll),
//!             DefStyle(style) => println!("received `def_style` from Xi core:\n{:?}", style),
//!             AvailablePlugins(plugins) => println!("received `available_plugins` from Xi core:\n{:?}", plugins),
//!             UpdateCmds(cmds) => println!("received `update_cmds` from Xi core:\n{:?}", cmds),
//!             PluginStarted(plugin) => println!("received `plugin_started` from Xi core:\n{:?}", plugin),
//!             PluginStoped(plugin) => println!("received `plugin_stoped` from Xi core:\n{:?}", plugin),
//!             ConfigChanged(config) => println!("received `config_changed` from Xi core:\n{:?}", config),
//!             ThemeChanged(theme) => println!("received `theme_changed` from Xi core:\n{:?}", theme),
//!             Alert(alert) => println!("received `alert` from Xi core:\n{:?}", alert),
//!             AvailableThemes(themes) => println!("received `available_themes` from Xi core:\n{:?}", themes),
//!             FindStatus(status) => println!("received `find_status` from Xi core:\n{:?}", status),
//!             ReplaceStatus(status) => println!("received `replace_status` from Xi core:\n{:?}", status),
//!             AvailableLanguages(langs) => println!("received `available_languages` from Xi core:\n{:?}", langs),
//!             LanguageChanged(lang) => println!("received `language_changed` from Xi core:\n{:?}", lang),
//!         }
//!         Box::new(future::ok(()))
//!     }
//!
//!     fn handle_measure_width(&mut self, request: MeasureWidth) -> ServerResult<Vec<Vec<f32>>> {
//!         Box::new(future::ok(Vec::new()))
//!     }
//! }
//!
//! struct MyFrontendBuilder;
//!
//! impl FrontendBuilder<MyFrontend> for MyFrontendBuilder {
//!     fn build(self, client: Client) -> MyFrontend {
//!         MyFrontend { client: client }
//!     }
//! }
//!
//! fn main() {
//!
//!     // spawn Xi core
//!     let (mut client, core_stderr) = spawn("xi-core", MyFrontendBuilder {});
//!
//!     // All clients must send client_started notification first
//!     tokio::run(client.client_started(None, None).map_err(|_|()));
//!     // start logging Xi core's stderr
//!     let log_core_errors = core_stderr
//!         .for_each(|msg| {
//!             println!("xi-core stderr: {}", msg);
//!             Ok(())
//!         })
//!         .map_err(|_| ());
//!
//!     ::std::thread::spawn(move || {
//!         tokio::run(log_core_errors);
//!     });
//!
//!     // Send a request to open a new view, and print the result
//!     let open_new_view = client
//!         .new_view(None)
//!         .map(|view_name| println!("opened new view: {}", view_name));
//!     tokio::run(open_new_view.map_err(|_| ()));
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
pub use crate::core::{spawn, CoreStderr};
pub use crate::errors::{ClientError, ServerError};
pub use crate::frontend::{Frontend, FrontendBuilder, XiNotification};
pub use crate::structs::{
    Alert, AvailableLanguages, AvailablePlugins, AvailableThemes, ConfigChanged, ConfigChanges,
    FindStatus, LanguageChanged, Line, MeasureWidth, ModifySelection, Operation, OperationType,
    PluginStarted, PluginStoped, Position, Query, ReplaceStatus, ScrollTo, Status, Style, StyleDef,
    ThemeChanged, ThemeSettings, Update, UpdateCmds, ViewId,
};
