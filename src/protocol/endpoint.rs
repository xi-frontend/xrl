use std::collections::HashMap;
use std::io;

use futures::sync::{mpsc, oneshot};
use futures::{Async, AsyncSink, Future, IntoFuture, Poll, Sink, StartSend, Stream};
use serde_json::Value;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_codec::{Decoder, Framed};

use super::codec::Codec;
use super::errors::RpcError;
use super::message::Response as ResponseMessage;
use super::message::{Message, Notification, Request};

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

pub trait Service: Send {
    type T: Into<Value> + Send + 'static;
    type E: Into<Value> + Send + 'static;
    type RequestFuture: IntoStaticFuture<Item = Self::T, Error = Self::E>;
    type NotificationFuture: IntoStaticFuture<Item = (), Error = ()>;

    fn handle_request(&mut self, method: &str, params: Value) -> Self::RequestFuture;

    fn handle_notification(&mut self, method: &str, params: Value) -> Self::NotificationFuture;
}

struct Server<S: Service + Send> {
    service: S,
    // This will receive responses from the service (or possibly from whatever worker tasks that
    // the service spawned). The u64 contains the id of the request that the response is for.
    pending_responses: mpsc::UnboundedReceiver<(u64, Result<S::T, S::E>)>,
    // We hand out a clone of this whenever we call `service.handle_request`.
    response_sender: mpsc::UnboundedSender<(u64, Result<S::T, S::E>)>,
}

unsafe impl<T: Service> Send for Server<T> {}

impl<S: Service> Server<S> {
    fn new(service: S) -> Self {
        let (tx, rx) = mpsc::unbounded();
        Server {
            service,
            pending_responses: rx,
            response_sender: tx,
        }
    }

    fn send_responses<T: AsyncRead + AsyncWrite>(
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

    fn process_request(&mut self, request: Request) {
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

    fn process_notification(&mut self, notification: Notification) {
        let Notification { method, params } = notification;
        let future = self.service.handle_notification(method.as_str(), params);
        // tokio::spawn returns a tokio::executor::Spawn that we don't
        // need so it's fine to ignore it.
        let _ = tokio::spawn(future.into_static_future());
    }
}

type ResponseTx = oneshot::Sender<Result<Value, Value>>;
/// Future response to a request. It resolved once the response is available.
pub struct Response(oneshot::Receiver<Result<Value, Value>>);

type AckTx = oneshot::Sender<()>;

/// A future that resolves when a notification has been effectively sent to the
/// server. It does not guarantees that the server receives it, just that it
/// has been sent.
pub struct Ack(oneshot::Receiver<()>);

type RequestTx = mpsc::UnboundedSender<(Request, ResponseTx)>;
type RequestRx = mpsc::UnboundedReceiver<(Request, ResponseTx)>;

type NotificationTx = mpsc::UnboundedSender<(Notification, AckTx)>;
type NotificationRx = mpsc::UnboundedReceiver<(Notification, AckTx)>;

impl Future for Response {
    type Item = Result<Value, Value>;
    type Error = RpcError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0
            .poll()
            .map_err(|oneshot::Canceled| RpcError::ResponseCanceled)
    }
}

impl Future for Ack {
    type Item = ();
    type Error = RpcError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0
            .poll()
            .map_err(|oneshot::Canceled| RpcError::AckCanceled)
    }
}

struct InnerClient {
    shutting_down: bool,
    request_id: u64,
    requests_rx: RequestRx,
    notifications_rx: NotificationRx,
    pending_requests: HashMap<u64, ResponseTx>,
    pending_notifications: Vec<AckTx>,
}

impl InnerClient {
    fn new() -> (Self, Client) {
        let (requests_tx, requests_rx) = mpsc::unbounded();
        let (notifications_tx, notifications_rx) = mpsc::unbounded();

        let client_proxy = Client::new(requests_tx, notifications_tx);

        let client = InnerClient {
            shutting_down: false,
            request_id: 0,
            requests_rx,
            notifications_rx,
            pending_requests: HashMap::new(),
            pending_notifications: Vec::new(),
        };

        (client, client_proxy)
    }

    fn shutdown(&mut self) {
        debug!("shutting down inner client");
        self.shutting_down = true;
    }

    fn is_shutting_down(&self) -> bool {
        self.shutting_down
    }

    fn process_notifications<T: AsyncRead + AsyncWrite>(&mut self, stream: &mut Transport<T>) {
        trace!("polling client notifications channel");
        loop {
            match self.notifications_rx.poll() {
                Ok(Async::Ready(Some((notification, ack_sender)))) => {
                    trace!("sending notification: {:?}", notification);
                    stream.send(Message::Notification(notification));
                    self.pending_notifications.push(ack_sender);
                }
                Ok(Async::NotReady) => {
                    trace!("no new notification from client");
                    break;
                }
                Ok(Async::Ready(None)) => {
                    warn!("client closed the notifications channel");
                    self.shutdown();
                }
                Err(()) => {
                    // I have no idea how this should be handled.
                    // The documentation does not tell what may trigger an error.
                    error!("an error occured while polling the notifications channel");
                    panic!("an error occured while polling the notifications channel");
                }
            }
        }
    }

    fn process_requests<T: AsyncRead + AsyncWrite>(&mut self, stream: &mut Transport<T>) {
        trace!("polling client requests channel");
        loop {
            match self.requests_rx.poll() {
                Ok(Async::Ready(Some((mut request, response_sender)))) => {
                    self.request_id += 1;
                    trace!("sending request: {:?}", request);
                    request.id = self.request_id;
                    stream.send(Message::Request(request));
                    self.pending_requests
                        .insert(self.request_id, response_sender);
                }
                Ok(Async::Ready(None)) => {
                    warn!("client closed the requests channel.");
                    self.shutdown();
                }
                Ok(Async::NotReady) => {
                    trace!("no new request from client");
                    break;
                }
                Err(()) => {
                    // I have no idea how this should be handled.
                    // The documentation does not tell what may trigger an error.
                    panic!("An error occured while polling the requests channel");
                }
            }
        }
    }

    fn process_response(&mut self, response: ResponseMessage) {
        if self.is_shutting_down() {
            return;
        }
        if let Some(response_tx) = self.pending_requests.remove(&response.id) {
            trace!("forwarding response to the client.");
            if let Err(e) = response_tx.send(response.result) {
                warn!("Failed to send response to client: {:?}", e);
            }
        } else {
            warn!("no pending request found for response {}", &response.id);
        }
    }

    fn acknowledge_notifications(&mut self) {
        for chan in self.pending_notifications.drain(..) {
            trace!("acknowledging notification.");
            if let Err(e) = chan.send(()) {
                warn!("Failed to send ack to client: {:?}", e);
            }
        }
    }
}

struct Transport<T: AsyncRead + AsyncWrite>(Framed<T, Codec>);

impl<T> Transport<T>
where
    T: AsyncRead + AsyncWrite,
{
    fn send(&mut self, message: Message) {
        debug!("sending message to remote peer: {:?}", message);
        match self.start_send(message) {
            Ok(AsyncSink::Ready) => return,
            // FIXME: there should probably be a retry mechanism.
            Ok(AsyncSink::NotReady(_message)) => panic!("The sink is full."),
            Err(e) => panic!("An error occured while trying to send message: {:?}", e),
        }
    }
}

impl<T> Stream for Transport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Item = Message;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll()
    }
}

impl<T> Sink for Transport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type SinkItem = Message;
    type SinkError = io::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.0.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.0.poll_complete()
    }
}

pub struct Endpoint<S: Service, T: AsyncRead + AsyncWrite> {
    stream: Transport<T>,
    client: InnerClient,
    server: Server<S>,
}

impl<S, T> Endpoint<S, T>
where
    S: Service,
    T: AsyncRead + AsyncWrite,
{
    pub fn new<B: ServiceBuilder<Service = S>>(stream: T, builder: B) -> (Self, Client) {
        let (client, client_proxy) = InnerClient::new();
        let endpoint = Endpoint {
            stream: Transport(Codec.framed(stream)),
            client: client,
            server: Server::new(builder.build(client_proxy.clone())),
        };
        (endpoint, client_proxy)
    }

    fn handle_message(&mut self, msg: Message) {
        debug!("handling message from remote peer {:?}", msg);
        use Message::*;
        match msg {
            Request(request) => self.server.process_request(request),
            Notification(notification) => self.server.process_notification(notification),
            Response(response) => self.client.process_response(response),
        }
    }

    fn flush(&mut self) {
        trace!("flushing stream");
        match self.stream.poll_complete() {
            Ok(Async::Ready(())) => self.client.acknowledge_notifications(),
            Ok(Async::NotReady) => return,
            Err(e) => panic!("Failed to flush the sink: {:?}", e),
        }
    }
}

impl<S, T: AsyncRead + AsyncWrite> Future for Endpoint<S, T>
where
    S: Service,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        trace!("polling stream");
        loop {
            match self.stream.poll()? {
                Async::Ready(Some(msg)) => self.handle_message(msg),
                Async::Ready(None) => {
                    warn!("stream closed by remote peer.");
                    return Ok(Async::Ready(()));
                }
                Async::NotReady => {
                    trace!("no new message in the stream");
                    break;
                }
            }
        }

        // Try to flush out all the responses that are queued up. If
        // this doesn't succeed, our output sink is full. In that
        // case, we apply some backpressure to our input stream by not
        // reading from it.
        //
        // Note that errors from poll_complete() are usually fatal,
        // hence the early return. See:
        // https://docs.rs/tokio/0.1.21/tokio/prelude/trait.Sink.html#errors-1
        if let Async::NotReady = self.server.send_responses(&mut self.stream)? {
            return Ok(Async::NotReady);
        }

        let mut client_shutdown = false;
        self.client.process_requests(&mut self.stream);
        self.client.process_notifications(&mut self.stream);
        if self.client.is_shutting_down() {
            warn!("Client shut down, exiting");
            client_shutdown = true;
        }

        self.flush();
        if client_shutdown {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

/// A `Service` builder. This trait must be implemented for servers.
pub trait ServiceBuilder {
    type Service: Service;

    fn build(self, client: Client) -> Self::Service;
}

#[derive(Clone)]
pub struct Client {
    requests_tx: RequestTx,
    notifications_tx: NotificationTx,
}

impl Client {
    fn new(requests_tx: RequestTx, notifications_tx: NotificationTx) -> Self {
        Client {
            requests_tx,
            notifications_tx,
        }
    }

    pub fn request(&self, method: &str, params: Value) -> Response {
        trace!(
            "forwarding request to endpoint (method={}, params={:?})",
            method,
            params
        );
        let request = Request {
            id: 0,
            method: method.to_owned(),
            params,
        };
        let (tx, rx) = oneshot::channel();
        // If send returns an Err, its because the other side has been dropped.
        // By ignoring it, we are just dropping the `tx`, which will mean the
        // rx will return Canceled when polled. In turn, that is translated
        // into a BrokenPipe, which conveys the proper error.
        let _ = mpsc::UnboundedSender::unbounded_send(&self.requests_tx, (request, tx));
        Response(rx)
    }

    pub fn notify(&self, method: &str, params: Value) -> Ack {
        trace!(
            "forwarding notification to endpoint (method={}, params={:?})",
            method,
            params
        );
        let notification = Notification {
            method: method.to_owned(),
            params,
        };
        let (tx, rx) = oneshot::channel();
        let _ = mpsc::UnboundedSender::unbounded_send(&self.notifications_tx, (notification, tx));
        Ack(rx)
    }
}

impl Future for Client {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::Ready(()))
    }
}
