use errors::ServerError;
use protocol::Service;
use futures::{future, Future};
use serde_json::{from_value, Value};
use structs::{ScrollTo, Style, Update};

pub type ServerResult<T> = Box<Future<Item = T, Error = ServerError>>;

pub trait Frontend {
    fn update(&mut self, update: Update) -> ServerResult<()>;
    fn scroll_to(&mut self, scroll_to: ScrollTo) -> ServerResult<()>;
    fn set_style(&mut self, style: Style) -> ServerResult<()>;
}

impl<F: Frontend> Service for F {
    type T = Value;
    type E = Value;
    type Error = ServerError;

    fn handle_request(
        &mut self,
        _method: &str,
        _params: Value,
    ) -> Box<Future<Item = Result<Self::T, Self::E>, Error = Self::Error>> {
        // AFAIK the core does not send any request to frontends yet
        // We should return an ServerError here
        unimplemented!();
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Box<Future<Item = (), Error = Self::Error>> {
        info!(
            "Handling notification: METHOD={}, PARAMS={}",
            method,
            &params
        );

        match method {
            "update" => match from_value::<Update>(params) {
                Ok(update) => self.update(update),
                Err(e) => {
                    error!("Can't handle notification: invalid parameters {}", &e);
                    Box::new(future::err(ServerError::DeserializeFailed(e)))
                }
            },

            "scroll_to" => match from_value::<ScrollTo>(params) {
                Ok(scroll_to) => self.scroll_to(scroll_to),
                Err(e) => {
                    error!("Can't handle notification: invalid parameters {}", &e);
                    Box::new(future::err(ServerError::DeserializeFailed(e)))
                }
            },

            "set_style" => match from_value::<Style>(params) {
                Ok(style) => self.set_style(style),
                Err(e) => {
                    error!("Can't handle notification: invalid parameters {}", &e);
                    Box::new(future::err(ServerError::DeserializeFailed(e)))
                }
            },

            _ => {
                error!("Can't handle notification: unknown method");
                Box::new(future::err(ServerError::UnknownMethod(method.into())))
            }
        }
    }
}
