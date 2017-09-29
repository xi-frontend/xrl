#![allow(unused_variables)]
extern crate env_logger;
extern crate futures;
extern crate tokio_core;
extern crate xrl;

use std::thread::sleep;
use std::time::Duration;

use futures::{future, Future};
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
    let mut client = spawn("xi-core", TestFrontend {}, &handle);

    sleep(Duration::from_millis(1000));
    let new_view_future = client
        .new_view(None)
        .map(|view_name| println!("{:?}", view_name));
    core.run(new_view_future).unwrap();
}
