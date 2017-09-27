use std::io;
use bytes::BytesMut;
use tokio_io::codec::{Decoder, Encoder};

use super::errors::DecodeError;
use super::message::Message;

pub struct Codec;

impl Decoder for Codec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        let res: Result<Option<Self::Item>, Self::Error>;
        let position = {
            let mut buf = io::Cursor::new(&src);
            loop {
                match Message::decode(&mut buf) {
                    Ok(message) => {
                        res = Ok(Some(message));
                        break;
                    }
                    Err(err) => match err {
                        DecodeError::Truncated => return Ok(None),
                        DecodeError::InvalidJson | DecodeError::InvalidMessage => continue,
                        DecodeError::Io(err) => {
                            res = Err(err);
                            break;
                        }
                    },
                }
            }
            buf.position() as usize
        };
        let _ = src.split_to(position);
        res
    }
}

impl Encoder for Codec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        let bytes = msg.to_vec();
        buf.extend_from_slice(&bytes);
        Ok(())
    }
}
