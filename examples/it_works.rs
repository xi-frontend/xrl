//! run with
//! `RUST_LOG=it_works=info,xrl=info cargo run --example it_works`
#![allow(unused_variables)]
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate tokio_core;
extern crate xrl;

use futures::{future, Future, Stream};
use tokio_core::reactor::Core;

use xrl::{spawn, Client, Frontend, FrontendBuilder, ScrollTo, ServerResult, Style, Update};


struct TestFrontend;

impl Frontend for TestFrontend {
    fn update(&mut self, update: Update) -> ServerResult<()> {
        Box::new(future::ok(()))
    }
    fn scroll_to(&mut self, scroll_to: ScrollTo) -> ServerResult<()> {
        Box::new(future::ok(()))
    }
    fn set_style(&mut self, style: Style) -> ServerResult<()> {
        Box::new(future::ok(()))
    }
}

impl FrontendBuilder<TestFrontend> for TestFrontend {
    fn build(self, _client: Client) -> TestFrontend {
        self
    }
}

fn main() {
    env_logger::init().unwrap();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let (mut client, core_stderr) = spawn("xi-core", TestFrontend {}, &handle);

    let log_core_errors = core_stderr
        .for_each(|msg| {
            warn!("xi-core stderr: {}", msg);
            Ok(())
        })
        .map_err(|_| ());
    core.handle().spawn(log_core_errors);

    let open_new_view = client
        .new_view(None)
        .map(|view_name| info!("opened new view: {}", view_name));
    core.run(open_new_view).unwrap();
}
