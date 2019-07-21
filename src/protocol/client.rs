use std::collections::HashMap;
use std::io;

use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Poll, Stream};
use serde_json::Value;
use tokio::io::{AsyncRead, AsyncWrite};

use super::errors::RpcError;
use super::message::Response as ResponseMessage;
use super::message::{Message, Notification, Request};
use super::transport::Transport;

type RequestRx = mpsc::UnboundedReceiver<(Request, ResponseTx)>;
type RequestTx = mpsc::UnboundedSender<(Request, ResponseTx)>;
type NotificationTx = mpsc::UnboundedSender<(Notification, AckTx)>;
type NotificationRx = mpsc::UnboundedReceiver<(Notification, AckTx)>;

type ResponseTx = oneshot::Sender<Result<Value, Value>>;
type AckTx = oneshot::Sender<()>;

/// Future response to a request. It resolved once the response is available.
pub struct Response(oneshot::Receiver<Result<Value, Value>>);

impl Future for Response {
    type Item = Result<Value, Value>;
    type Error = RpcError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0
            .poll()
            .map_err(|oneshot::Canceled| RpcError::ResponseCanceled)
    }
}

/// A future that resolves when a notification has been effectively sent to the
/// server. It does not guarantees that the server receives it, just that it
/// has been sent.
pub struct Ack(oneshot::Receiver<()>);

impl Future for Ack {
    type Item = ();
    type Error = RpcError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0
            .poll()
            .map_err(|oneshot::Canceled| RpcError::AckCanceled)
    }
}

pub struct InnerClient {
    shutting_down: bool,
    request_id: u64,
    requests_rx: RequestRx,
    notifications_rx: NotificationRx,
    pending_requests: HashMap<u64, ResponseTx>,
    pending_notifications: Vec<AckTx>,
    shutdown_rx: mpsc::UnboundedReceiver<()>,
}

impl InnerClient {
    pub fn new() -> (Self, Client) {
        let (requests_tx, requests_rx) = mpsc::unbounded();
        let (notifications_tx, notifications_rx) = mpsc::unbounded();
        let (shutdown_tx, shutdown_rx) = mpsc::unbounded();

        let client_proxy = Client::new(requests_tx, notifications_tx, shutdown_tx);

        let client = InnerClient {
            shutting_down: false,
            request_id: 0,
            requests_rx,
            notifications_rx,
            pending_requests: HashMap::new(),
            pending_notifications: Vec::new(),
            shutdown_rx,
        };

        (client, client_proxy)
    }

    pub fn shutdown(&mut self) {
        debug!("shutting down inner client");
        self.shutting_down = true;
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutting_down
    }

    pub fn process_shutdown_signals(&mut self) {
        trace!("polling shutdown signal channel");
        loop {
            match self.shutdown_rx.poll() {
                Ok(Async::Ready(Some(()))) => {
                    info!("Received shutdown signal");
                    self.shutdown();
                    // Note that in theory, we should continue polling
                    // until NotReady, but since we're shutting down
                    // anyway, the Endpoint is going to be dropped so
                    // it does not matter if the rest of the IO events
                    // are being polled or not.
                    break;
                }
                Ok(Async::Ready(None)) => {
                    warn!("client closed the shutdown signal channel");
                    self.shutdown();
                    break;
                }
                Ok(Async::NotReady) => {
                    trace!("no shutdown signal from client");
                    break;
                }
                Err(()) => {
                    error!("an error occured while polling the shutdown signal channel");
                    panic!("an error occured while polling the shutdown signal channel");
                }
            }
        }
    }

    pub fn process_notifications<T: AsyncRead + AsyncWrite>(&mut self, stream: &mut Transport<T>) {
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
                    break;
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

    pub fn process_requests<T: AsyncRead + AsyncWrite>(&mut self, stream: &mut Transport<T>) {
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
                    break;
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

    pub fn process_response(&mut self, response: ResponseMessage) {
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

    pub fn acknowledge_notifications(&mut self) {
        for chan in self.pending_notifications.drain(..) {
            trace!("acknowledging notification.");
            if let Err(e) = chan.send(()) {
                warn!("Failed to send ack to client: {:?}", e);
            }
        }
    }
}

/// `Client` can be used to send Xi-RPC requests and notifications. It
/// implements `Clone` so multiple clients can be instantiated. When
/// all the `Client` instances are dropped, the Xi-RPC endoint shuts
/// down. If the Xi-RPC endpoint shuts down while there are still
/// `Client` instances, `Client::request()`, `Client::notify()` and
/// `Client::shutdown()` can still be called on these instances, but
/// will have no effect.
#[derive(Clone)]
pub struct Client {
    requests_tx: RequestTx,
    notifications_tx: NotificationTx,
    shutdown_tx: mpsc::UnboundedSender<()>,
}

impl Client {
    fn new(
        requests_tx: RequestTx,
        notifications_tx: NotificationTx,
        shutdown_tx: mpsc::UnboundedSender<()>,
    ) -> Self {
        Client {
            requests_tx,
            notifications_tx,
            shutdown_tx,
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

    /// Forces the Xi-RPC endpoint to shut down. After this, the the
    /// `request()`, `notify()` and `shutdown()` methods can still be
    /// called but will have not effect.
    pub fn shutdown(&self) {
        let _ = mpsc::UnboundedSender::unbounded_send(&self.shutdown_tx, ());
    }
}

impl Future for Client {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::Ready(()))
    }
}
