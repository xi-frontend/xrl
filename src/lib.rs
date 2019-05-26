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
//!     fn handle_event(&mut self, ev: XiEvent) -> ServerResult<()> {
//!         match ev {
//!             XiEvent::Update(update) => println!("received `update` from Xi core:\n{:?}", update),
//!             XiEvent::ScrollTo(scroll) => println!("received `scroll_to` from Xi core:\n{:?}", scroll),
//!             XiEvent::DefStyle(style) => println!("received `def_style` from Xi core:\n{:?}", style),
//!             XiEvent::AvailablePlugins(plugins) => println!("received `available_plugins` from Xi core:\n{:?}", plugins),
//!             XiEvent::UpdateCmds(cmds) => println!("received `update_cmds` from Xi core:\n{:?}", cmds),
//!             XiEvent::PluginStarted(plugin) => println!("received `plugin_started` from Xi core:\n{:?}", plugin),
//!             XiEvent::PluginStoped(plugin) => println!("received `plugin_stoped` from Xi core:\n{:?}", plugin),
//!             XiEvent::ConfigChanged(config) => println!("received `config_changed` from Xi core:\n{:?}", config),
//!             XiEvent::ThemeChanged(theme) => println!("received `theme_changed` from Xi core:\n{:?}", theme),
//!         }
//!         Box::new(future::ok(()))
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
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", deny(clippy))]
#![cfg_attr(feature = "clippy", allow(missing_docs_in_private_items))]
#![cfg_attr(feature = "clippy", allow(type_complexity))]

extern crate bytes;
extern crate futures;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio;
extern crate tokio_process;
extern crate tokio_codec;
extern crate syntect;

mod protocol;
mod client;
mod errors;
mod structs;
mod frontend;
mod core;
mod cache;

pub use cache::LineCache;
pub use frontend::{XiEvent, Frontend, FrontendBuilder, ServerResult};
pub use client::{Client, ClientResult};
pub use errors::{ClientError, ServerError};
pub use core::{spawn, CoreStderr};
pub use structs::{
    AvailablePlugins, PluginStarted, PluginStoped, ThemeChanged,
    ThemeSettings,
    UpdateCmds, ConfigChanged, ConfigChanges, ScrollTo, Position,
    Update, Style, Operation, OperationType, Line, StyleDef,
    ViewId, ModifySelection,
};
