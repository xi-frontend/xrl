use std::io;

use futures::task::Context;
use futures::{Sink, Stream};
use futures_core::task::Poll;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_codec::{Decoder, Framed};

use super::codec::Codec;
use super::message::Message;

pub struct Transport<T: AsyncRead + AsyncWrite>(Framed<T, Codec>);

impl<T> Transport<T>
where
    T: AsyncRead + AsyncWrite,
{
    pub fn new(stream: T) -> Self {
        Transport(Codec.framed(stream))
    }

    pub fn send(&mut self, message: Message) {
        debug!("sending message to remote peer: {:?}", message);
        match self.start_send(message) {
            Ok(Poll::Ready(Ok())) => (),
            // FIXME: there should probably be a retry mechanism.
            Ok(Poll::NotReady(_message)) => panic!("The sink is full."),
            Err(e) => panic!("An error occured while trying to send message: {:?}", e),
        }
    }
}

impl<T> Stream for Transport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Item = Message;

    fn poll_next(&mut self, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.0.poll()
    }
}

impl<T> Sink<Message> for Transport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Error = io::Error;

    fn start_send(&mut self, item: Message) -> Result<Message, Self::Error> {
        self.0.start_send(item)
    }

    fn poll_close(&mut self) -> Poll<Result<(), Self::Error>> {
        self.0.poll_close()
    }
}
