//! # `xrl`
//! Crate for working with the Xi frontend protocol, with an optional
//! tokio based client.
//!
//! xrl is split into three modules `protocol` contains structs that represent the xi-rpc
//! protocol. The `client` module contains objects for starting the xi-core client either through
//! the command line or a seperate thread. The `editor` module contains structures that can be used
//! to store & handle messages from xi-core.
//!
//! ### XiCore
//! Xi Core can be loaded calling either an executable located in $PATH or by launching
//! the xi-core-lib in a seperate thread. This can be specified by passing the
//! [`XiLocation`](./enum.XiLocation.html) enum to the client struct.
//! The `XiLocation::Embeded` Represents XiCore running in a seperate thread
//! using the xi-core-lib interface and the `XiLocation::Path { .. }` will attempt to launch xi-core
//! using [`Command`](https://docs.rs/tokio/0.2.21/tokio/process/struct.Command.html). Errors will
//! be propagated through the `Message::Error` variant.
//!
//! ### Protocol
//! The [`protocol`](./protocol/index.html) module contains the xi frontend protocol.
//! The main Object is the [`Message`](./protocol/enum.Message.html) enum.
//! It can be serialized using the [`serde_json`](https://docs.rs/serde_json) crate.
//!
//! ### Client
//! **Requires** [`client`](./client/index.html) **feature** (enabled by default)
//! the client can be created with the `Client::new` method and methods for interacting
//! with the editor can be used by importing the [`ClientExt`](./client/trait.ClientExt.html) trait.
//!
//! ### API
//! **Requires** [`api`](./api/index.html) **feature** (enabled by default)
//! This module contains common structures for building an editor with xi.
//! Most can be used indevidually or in conjunction with other objects in the api module.
//!   - **Editor**: Implements a basic xi client editor with most features implemented.
//!   - **LineCache**: A basic linecache implementation to manage the view line cache.
//!   - **StyleCache**: Style Cache to hold styles received from xi-core.
//!   - **ViewList**: Store currently open views from xi.
//!   - **View**: Store information related to a particular view.
//!   - **ViewPort**: Visible window into the line cache.
//!
//! ### Testing
//! The [`TestClient`](./struct.TestClient.html) is used for testing this library and my prove
//! useful when testing frontend components. It has the same API as the Client struct with extra
//! `fail_on_error` & `check-response` methods to set whether receiving an error from xi should
//! cause an error and to wait for a certain response respectivily.
//!
//! ### Examples
//!
//! #### Parse Xi Rpc
//! ```should_panic rust
//!    use xrl::protocol::Message;
//!
//!    let data = "XI-RPC";
//!    let _msg = serde_json::from_str::<Message>(data).unwrap();
//! ```
//!
//! #### Create a new client
//! ```rust
//!    use xrl::XiLocation;
//!    use xrl::client::Client;
//!
//!    let _client = Client::new(XiLocation::Embeded).unwrap();
//! ```
//!

#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "client")]
pub mod client;
pub mod protocol;

mod location;
pub use self::location::XiLocation;

#[cfg(feature = "client")]
mod test_client;
#[cfg(feature = "client")]
pub use self::test_client::TestClient;
