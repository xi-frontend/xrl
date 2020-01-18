use std::io;

use futures::{Future, Sink, Stream};
use futures_core::task::Poll;
use tokio::io::{AsyncRead, AsyncWrite};

use super::client::Client;
use super::client::InnerClient;
use super::message::Message;
use super::server::{Server, Service, ServiceBuilder};
use super::transport::Transport;

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
            stream: Transport::new(stream),
            server: Server::new(builder.build(client_proxy.clone())),
            client,
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
            Ok(Poll::Ready(())) => self.client.acknowledge_notifications(),
            Ok(Poll::NotReady) => (),
            Err(e) => panic!("Failed to flush the sink: {:?}", e),
        }
    }
}

impl<S, T: AsyncRead + AsyncWrite> Future for Endpoint<S, T>
where
    S: Service,
{
    type Output = Result<(), io::Error>;

    fn poll(&mut self) -> Poll<Self::Output> {
        trace!("polling stream");
        loop {
            match self.stream.poll()? {
                Poll::Ready(Some(msg)) => self.handle_message(msg),
                Poll::Ready(None) => {
                    warn!("stream closed by remote peer.");
                    return Ok(Poll::Ready(()));
                }
                Poll::NotReady => {
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
        if let Poll::NotReady = self.server.send_responses(&mut self.stream)? {
            return Ok(Poll::NotReady);
        }

        let mut client_shutdown = false;
        self.client.process_requests(&mut self.stream);
        self.client.process_notifications(&mut self.stream);
        self.client.process_shutdown_signals();
        if self.client.is_shutting_down() {
            warn!("Client shut down, exiting");
            client_shutdown = true;
        }

        self.flush();
        if client_shutdown {
            Ok(Poll::Ready(()))
        } else {
            Ok(Poll::NotReady)
        }
    }
}
