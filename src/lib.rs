//! `xrl` is a crate for working with the Xi frontend protocol, with an optional
//! tokio based client.
//!
//! xrl is split into three modules the `protocol` contains structs that represent the xi-rpc
//! protocol. The `client` module contains objects for starting the xi-core client either through
//! the command line or a seperate thread. The `editor` module contains structures that can be used
//! to store & handle messages from xi-core.
//!
//! ### XiCore
//! Xi Core can be loaded calling either an executable located in $PATH or by launching
//! the xi-core-lib in a seperate thread. This can be specified by passing the `XiLocation` enum to
//! the client struct.
//!
//! ### Parse Xi Rpc
//! ```should_panic rust
//!    use xrl::protocol::Message;
//!
//!    let data = "XI-RPC";
//!    let _msg = serde_json::from_str::<Message>(data).unwrap();
//! ```
//!
//! ### Create a new client
//! ```rust
//!    use xrl::XiLocation;
//!    use xrl::client::Client;
//!
//!    let _client = Client::new(XiLocation::Embeded).unwrap();
//! ```
//!

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
