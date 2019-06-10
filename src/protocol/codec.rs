use bytes::{BufMut, BytesMut};
use std::io;
use tokio_codec::{Decoder, Encoder};

use super::errors::DecodeError;
use super::message::Message;

pub struct Codec;

impl Decoder for Codec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if let Some(n) = buf.as_ref().iter().position(|b| *b == b'\n') {
            let line = buf.split_to(n);
            trace!("<<< {}", ::std::str::from_utf8(&line).unwrap());
            buf.split_to(1); // remove the '\n'

            match Message::decode(&mut io::Cursor::new(&line)) {
                Ok(message) => return Ok(Some(message)),
                Err(err) => match err {
                    DecodeError::Io(err) => return Err(err),
                    _ => return Ok(None),
                },
            }
        }
        Ok(None)
    }
}

impl Encoder for Codec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        let bytes = msg.to_vec();
        trace!(">>> {}", ::std::str::from_utf8(&bytes).unwrap());
        buf.reserve(bytes.len() + 1);
        buf.put_slice(&bytes);
        buf.put(b'\n');
        Ok(())
    }
}
