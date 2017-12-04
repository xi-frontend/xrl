#![allow(unused_variables)]
extern crate futures;
extern crate tokio_core;
extern crate xrl;

use futures::{future, Future, Stream};
use tokio_core::reactor::Core;
use xrl::{
    Update, ScrollTo, Style,
    AvailablePlugins, UpdateCmds,
    PluginStarted, PluginStoped,
    ConfigChanged, ThemeChanged,
    Client, ServerResult, Frontend,
    FrontendBuilder, spawn,
};


// Type that represent our client
struct MyFrontend {
    #[allow(dead_code)]
    client: Client,
}

// Implement how our client handles notifications and requests from the core.
impl Frontend for MyFrontend {
    fn update(&mut self, update: Update) -> ServerResult<()> {
        println!("received `update` from Xi core:\n{:?}", update);
        // note that we could send requests/notifications to the core here with `self.client`
        Box::new(future::ok(()))
    }
    fn scroll_to(&mut self, scroll_to: ScrollTo) -> ServerResult<()> {
        println!("received `scroll_to` from Xi core:\n{:?}", scroll_to);
        Box::new(future::ok(()))
    }
    fn def_style(&mut self, style: Style) -> ServerResult<()> {
        println!("received `def_style` from Xi core:\n{:?}", style);
        Box::new(future::ok(()))
    }
    fn available_plugins(&mut self, scroll_to: AvailablePlugins) -> ServerResult<()> {
        println!("received `available_plugins` from Xi core:\n{:?}", scroll_to);
        Box::new(future::ok(()))
    }
    fn update_cmds(&mut self, style: UpdateCmds) -> ServerResult<()> {
        println!("received `update_cmds` from Xi core:\n{:?}", style);
        Box::new(future::ok(()))
    }
    fn plugin_started(&mut self, style: PluginStarted) -> ServerResult<()> {
        println!("received `plugin_started` from Xi core:\n{:?}", style);
        Box::new(future::ok(()))
    }
    fn plugin_stoped(&mut self, style: PluginStoped) -> ServerResult<()> {
        println!("received `plugin_stoped` from Xi core:\n{:?}", style);
        Box::new(future::ok(()))
    }
    fn config_changed(&mut self, style: ConfigChanged) -> ServerResult<()> {
        println!("received `config_changed` from Xi core:\n{:?}", style);
        Box::new(future::ok(()))
    }
    fn theme_changed(&mut self, style: ThemeChanged) -> ServerResult<()> {
        println!("received `theme_changed` from Xi core:\n{:?}", style);
        Box::new(future::ok(()))
    }
}

struct MyFrontendBuilder;

impl FrontendBuilder<MyFrontend> for MyFrontendBuilder {
    fn build(self, client: Client) -> MyFrontend {
        MyFrontend { client: client }
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // spawn Xi core
    let (mut client, core_stderr) = spawn("xi-core", MyFrontendBuilder {}, &handle);

    // start logging Xi core's stderr
    let log_core_errors = core_stderr
        .for_each(|msg| {
            println!("xi-core stderr: {}", msg);
            Ok(())
        })
        .map_err(|_| ());
    core.handle().spawn(log_core_errors);

    // Send a request to open a new view, and print the result
    let open_new_view = client
        .new_view(None)
        .map(|view_name| println!("opened new view: {}", view_name));
    core.run(open_new_view).unwrap();
}
