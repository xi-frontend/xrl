use errors::RpcError;
use protocol::Service;
use futures::{future, Future};
use serde_json::{from_value, Value};
use structs::{Position, Style, Update};


pub trait Frontend {
    fn update(&mut self, update: Update) -> Box<Future<Item = (), Error = RpcError>>;
    fn scroll_to(&mut self, line: u64, column: u64) -> Box<Future<Item = (), Error = RpcError>>;
    fn set_style(&mut self, style: Style) -> Box<Future<Item = (), Error = RpcError>>;
}

impl<F: Frontend> Service for F {
    type T = Value;
    type E = Value;
    type Error = RpcError;

    fn handle_request(
        &mut self,
        _method: &str,
        _params: Value,
    ) -> Box<Future<Item = Result<Self::T, Self::E>, Error = Self::Error>> {
        // AFAIK the core does not send any request to frontends yet
        // We should return an RpcError here
        unimplemented!();
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Box<Future<Item = (), Error = Self::Error>> {
        match method {
            "update" => match from_value::<Update>(params) {
                Ok(update) => self.update(update),
                Err(_) => Box::new(future::err(RpcError::InvalidParameters)),
            },
            "scroll_to" => match from_value::<Position>(params) {
                Ok(position) => self.scroll_to(position.0, position.1),
                Err(_) => Box::new(future::err(RpcError::InvalidParameters)),
            },
            "set_style" => match from_value::<Style>(params) {
                Ok(style) => self.set_style(style),
                Err(_) => Box::new(future::err(RpcError::InvalidParameters)),
            },
            _ => {
                unimplemented!();
            }
        }
    }
}
