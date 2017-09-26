#![allow(dead_code)]
use std::collections::hash_map::HashMap;
use std::io::BufReader;
use std::io::prelude::*;
use std::process::ChildStdin;
use std::process::Command;
use std::process::Stdio;

use structs::{Update, Style, Position};

struct Stream {
    stdout: ChildStdout,
    stdin: ChildStdin,
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.stdout.read(buf)
    }
}

impl AsyncRead for Stream {
    // FIXME: do I actually have to implement this?
    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [u8]) -> bool {
        self.stdout.prepare_uninitialized_buffer(buf)
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.stdin.write(buf)
    }
}

impl AsyncWrite for Stream {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        self.stdin.shutdown()
    }
}

pub trait Frontend {
    fn update<T, E>(update: Update) -> Box<Future<Item = (), Error = ()>>;
    fn scroll_to(position: Position) -> Box<Future<Item = (), Error = ()>>;
    fn set_style(style: Style) -> Box<Future<Item = (), Error = ()>>;
}

impl<F: Frontend> Service for F {
    type T = Value;
    type E = Value;
    type Error = String;

    fn handle_request(&mut self, method: &str, params: Value) -> Box<Future<Item = Result<Self::T, Self::E>, Error = Self::Error>> {
        // AFAIK the core does not send any request to frontends yet
        Box::new(future::ok(Err(to_value("The frontend does not handle requests"))));
    }

    fn handle_notification(&mut self, method: &str, params: Value) -> Box<Future<Item = (), Error = Self::Error>> {
        match method {
            "update" => self.update(from_value::<Update>(params)),
            "scroll_to" => self.scroll_to(from_value::<Position>(params)),
            "set_style" => self.set_style(from_value::<Style>(params)),
            _ =>  {
                error!("Unknown method {:?}.", method);
                Err(format!("Unknown method {:?}", method))
            }
        }
    }
}
