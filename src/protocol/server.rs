use std::io;

use futures::sync::mpsc;
use futures::{Async, Future, IntoFuture, Poll, Sink, Stream};
use serde_json::Value;
use tokio::io::{AsyncRead, AsyncWrite};

use super::client::Client;
use super::message::Response as ResponseMessage;
use super::message::{Message, Notification, Request};
use super::transport::Transport;

pub trait Service: Send {
    type T: Into<Value> + Send + 'static;
    type E: Into<Value> + Send + 'static;
    type RequestFuture: IntoStaticFuture<Item = Self::T, Error = Self::E>;
    type NotificationFuture: IntoStaticFuture<Item = (), Error = ()>;

    fn handle_request(&mut self, method: &str, params: Value) -> Self::RequestFuture;

    fn handle_notification(&mut self, method: &str, params: Value) -> Self::NotificationFuture;
}

pub struct Server<S: Service + Send> {
    service: S,
    // This will receive responses from the service (or possibly from whatever worker tasks that
    // the service spawned). The u64 contains the id of the request that the response is for.
    pending_responses: mpsc::UnboundedReceiver<(u64, Result<S::T, S::E>)>,
    // We hand out a clone of this whenever we call `service.handle_request`.
    response_sender: mpsc::UnboundedSender<(u64, Result<S::T, S::E>)>,
}

unsafe impl<T: Service> Send for Server<T> {}

impl<S: Service> Server<S> {
    pub fn new(service: S) -> Self {
        let (tx, rx) = mpsc::unbounded();
        Server {
            service,
            pending_responses: rx,
            response_sender: tx,
        }
    }

    pub fn send_responses<T: AsyncRead + AsyncWrite>(
        &mut self,
        sink: &mut Transport<T>,
    ) -> Poll<(), io::Error> {
        trace!("Server: flushing responses");
        while let Ok(poll) = self.pending_responses.poll() {
            if let Async::Ready(Some((id, result))) = poll {
                let msg = Message::Response(ResponseMessage {
                    id,
                    result: result.map(Into::into).map_err(Into::into),
                });
                // FIXME: in futures 0.2, use poll_ready before reading from pending_responses, and
                // don't panic here.
                sink.start_send(msg).unwrap();
            } else {
                if let Async::Ready(None) = poll {
                    panic!("we store the sender, it can't be dropped");
                }

                // We're done pushing all messages into the sink, now try to flush it.
                return sink.poll_complete();
            }
        }
        panic!("an UnboundedReceiver should never give an error");
    }

    pub fn process_request(&mut self, request: Request) {
        let Request { method, params, id } = request;
        let response_sender = self.response_sender.clone();
        let future = self
            .service
            .handle_request(method.as_str(), params)
            .into_static_future()
            .then(move |response| {
                // Send the service's response back to the Server, so
                // that it can be sent over the transport layer.
                //
                // TODO: handle error from unbounded_send?
                response_sender
                    .unbounded_send((id, response))
                    .map_err(|_| ())
            });
        // tokio::spawn returns a tokio::executor::Spawn that we don't
        // need so it's fine to ignore it.
        let _ = tokio::spawn(future);
    }

    pub fn process_notification(&mut self, notification: Notification) {
        let Notification { method, params } = notification;
        let future = self.service.handle_notification(method.as_str(), params);
        // tokio::spawn returns a tokio::executor::Spawn that we don't
        // need so it's fine to ignore it.
        let _ = tokio::spawn(future.into_static_future());
    }
}

/// A `Service` builder. This trait must be implemented for servers.
pub trait ServiceBuilder {
    type Service: Service;

    fn build(self, client: Client) -> Self::Service;
}

// We need this IntoStaticFuture trait because the future we spawn on
// Tokio's event loop must have the 'static lifetime.and be Send.

/// Class of types which can be converted into a future. This trait is
/// only differs from
/// [`futures::future::IntoFuture`](https://docs.rs/futures/0.1.17/futures/future/trait.IntoFuture.html)
/// in that the returned future has the `'static` lifetime.
pub trait IntoStaticFuture {
    /// The future that this type can be converted into.
    type Future: Future<Item = Self::Item, Error = Self::Error> + 'static + Send;
    /// The item that the future may resolve with.
    type Item;
    /// The error that the future may resolve with.
    type Error;

    /// Consumes this object and produces a future.
    fn into_static_future(self) -> Self::Future;
}

impl<F: IntoFuture> IntoStaticFuture for F
where
    <F as IntoFuture>::Future: 'static + Send,
{
    type Future = <F as IntoFuture>::Future;
    type Item = <F as IntoFuture>::Item;
    type Error = <F as IntoFuture>::Error;

    fn into_static_future(self) -> Self::Future {
        self.into_future()
    }
}
