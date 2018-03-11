use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::io;

use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend, Stream};
use futures::sync::{mpsc, oneshot};
use tokio_io::codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};
use serde_json::Value;

use super::message::{Message, Notification, Request};
use super::message::Response as ResponseMessage;
use super::codec::Codec;
use super::errors::RpcError;

pub trait Service {
    type Error: Error;
    type T: Into<Value>;
    type E: Into<Value>;

    fn handle_request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Box<Future<Item = Result<Self::T, Self::E>, Error = Self::Error>>;

    fn handle_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Box<Future<Item = (), Error = Self::Error>>;
}

struct Server<S: Service> {
    service: S,
    request_tasks: HashMap<u64, Box<Future<Item = Result<S::T, S::E>, Error = S::Error>>>,
    notification_tasks: Vec<Box<Future<Item = (), Error = S::Error>>>,
}

impl<S: Service> Server<S> {
    fn new(service: S) -> Self {
        Server {
            service,
            request_tasks: HashMap::new(),
            notification_tasks: Vec::new(),
        }
    }

    fn poll_notification_tasks(&mut self) {
        trace!("polling pending notification tasks");
        let mut done = vec![];
        for (idx, task) in self.notification_tasks.iter_mut().enumerate() {
            match task.poll() {
                Ok(Async::Ready(_)) => done.push(idx),
                Ok(Async::NotReady) => continue,
                Err(e) => {
                    done.push(idx);
                    error!("failed to handle notification: {}", e);
                }
            }
        }
        for idx in done.iter().rev() {
            self.notification_tasks.remove(*idx);
        }
    }

    fn poll_request_tasks<T: AsyncRead + AsyncWrite>(&mut self, stream: &mut Transport<T>) {
        trace!("polling pending requests");
        let mut done = vec![];
        for (id, task) in &mut self.request_tasks {
            match task.poll() {
                Ok(Async::Ready(response)) => {
                    let msg = Message::Response(ResponseMessage {
                        id: *id,
                        result: response.map(|v| v.into()).map_err(|e| e.into()),
                    });
                    done.push(*id);
                    stream.send(msg);
                }
                Ok(Async::NotReady) => continue,
                Err(e) => {
                    done.push(*id);
                    error!("Failed to handle request: {}", e);
                }
            }
        }

        for idx in done.iter_mut().rev() {
            let _ = self.request_tasks.remove(idx);
        }
    }

    fn process_request(&mut self, request: Request) {
        let method = request.method.as_str();
        let params = request.params;
        let response = self.service.handle_request(method, params);
        self.request_tasks.insert(request.id, response);
    }

    fn process_notification(&mut self, notification: Notification) {
        let method = notification.method.as_str();
        let params = notification.params;
        let task = self.service.handle_notification(method, params);
        self.notification_tasks.push(task);
    }
}

type ResponseTx = oneshot::Sender<Result<Value, Value>>;
/// Future response to a request. It resolved once the response is available.
pub struct Response(oneshot::Receiver<Result<Value, Value>>);

type AckTx = oneshot::Sender<()>;

/// A future that resolves when a notification has been effictively sent to the
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

pub struct Endpoint<S: Service, T: AsyncRead + AsyncWrite> {
    stream: RefCell<Transport<T>>,
    client: Option<RefCell<InnerClient>>,
    server: Option<RefCell<Server<S>>>,
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

impl<S, T> Endpoint<S, T>
where
    S: Service,
    T: AsyncRead + AsyncWrite,
{
    pub fn new(stream: T) -> Self {
        Endpoint {
            stream: RefCell::new(Transport(stream.framed(Codec))),
            client: None,
            server: None,
        }
    }

    pub fn set_server(&mut self, service: S) {
        self.server = Some(RefCell::new(Server::new(service)));
    }

    pub fn set_client(&mut self) -> Client {
        let (client, client_proxy) = InnerClient::new();
        self.client = Some(RefCell::new(client));
        client_proxy
    }

    fn handle_message(&mut self, msg: Message) {
        debug!("handling message from remote peer {:?}", msg);
        match msg {
            Message::Request(request) => if let Some(ref mut server) = self.server {
                server.get_mut().process_request(request);
            } else {
                warn!(
                    "this endpoint does not handle requests => request ignored: {:?}",
                    request
                );
            },
            Message::Notification(notification) => if let Some(ref mut server) = self.server {
                server.get_mut().process_notification(notification);
            } else {
                warn!(
                    "this endpoint does not handle notifications => notification ignored: {:?}",
                    notification
                );
            },
            Message::Response(response) => if let Some(ref mut client) = self.client {
                client.get_mut().process_response(response);
            } else {
                warn!(
                    "this endpoint does not handle responses => response ignored: {:?}",
                    response
                );
            },
        }
    }

    fn flush(&mut self) {
        trace!("flushing stream");
        match self.stream.get_mut().poll_complete() {
            Ok(Async::Ready(())) => if let Some(ref mut client) = self.client {
                client.get_mut().acknowledge_notifications();
            },
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
            match self.stream.get_mut().poll()? {
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

        if let Some(ref mut server) = self.server {
            let server = server.get_mut();
            server.poll_request_tasks(self.stream.get_mut());
            server.poll_notification_tasks();
        }

        let mut client_shutdown: bool = false;
        if let Some(ref mut client) = self.client {
            let client = client.get_mut();
            let stream = self.stream.get_mut();
            client.process_requests(stream);
            client.process_notifications(stream);
            if client.is_shutting_down() {
                warn!("Client shut down, exiting");
                client_shutdown = true;
            }
        }
        if client_shutdown {
            self.client = None;
        }

        self.flush();
        Ok(Async::NotReady)
    }
}

/// A `Service` builder. This trait must be implemented for servers.
pub trait ServiceBuilder {
    type Service: Service + 'static;

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
