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
extern crate tokio_io;

mod core;
mod client;
mod errors;
mod structs;
mod frontend;

pub use frontend::Frontend;
pub use client::{Client, RpcResult};
pub use errors::RpcError;
