extern crate env_logger;
extern crate futures;
extern crate tokio_core;
extern crate xi_rpc_tokio;

use futures::{future, Future};
use tokio_core::reactor::Core;
use xi_rpc_tokio::{spawn, Frontend, RpcResult, Style, Update};
use std::thread::sleep;
use std::time::Duration;


struct TestFrontend;

impl Frontend for TestFrontend {
    fn update(&mut self, update: Update) -> RpcResult<()> {
        println!("Got update from core: {:?}", update);
        Box::new(future::ok(()))
    }
    fn scroll_to(&mut self, line: u64, column: u64) -> RpcResult<()> {
        println!("Got scroll_to from core: ({} {})", line, column);
        Box::new(future::ok(()))
    }
    fn set_style(&mut self, style: Style) -> RpcResult<()> {
        println!("Got set_tyle from core: {:?}", style);
        Box::new(future::ok(()))
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
