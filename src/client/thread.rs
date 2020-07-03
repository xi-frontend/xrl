use log::trace;
use serde_json::Value;
use xi_core_lib::XiCore;
use xi_rpc::RpcLoop;

use std::io::Error as IoError;
use std::io::Result as IoResult;
use std::io::{BufReader, ErrorKind, Read, Write};
use std::thread::spawn;
use tokio::sync::mpsc::{
    unbounded_channel as channel, UnboundedReceiver as Receiver, UnboundedSender as Sender,
};

use crate::client::ClientImpl;
use crate::protocol::Message;

pub struct Thread {
    request_id: usize,
    stdout_rx: Receiver<Value>,
    stdin_tx: Sender<Value>,
}

impl Thread {
    pub fn new() -> IoResult<Thread> {
        let (stdin_tx, stdin_rx) = channel();
        let (stdout_tx, stdout_rx) = channel();

        let mut editor = XiCore::new();
        let mut rpc_loop = RpcLoop::new(XiWriter(stdout_tx));
        spawn(move || rpc_loop.mainloop(|| BufReader::new(XiReader(stdin_rx)), &mut editor));

        Ok(Thread {
            request_id: 0,
            stdout_rx,
            stdin_tx,
        })
    }
}

#[async_trait::async_trait]
impl ClientImpl for Thread {
    fn next_id(&mut self) -> usize {
        self.request_id += 1;
        self.request_id - 1
    }

    async fn receive(&mut self) -> IoResult<Message> {
        match self.stdout_rx.recv().await {
            Some(value) => {
                trace!("client < xi-core: {}", value);
                Ok(serde_json::from_value(value)?)
            }
            None => Err(IoError::new(
                ErrorKind::InvalidInput,
                "Failed to read from Xi Core",
            )),
        }
    }

    async fn send(&mut self, msg: Value) -> IoResult<()> {
        trace!("client > xi-core: {:?}", msg);
        self.stdin_tx
            .send(msg)
            .map_err(|err| IoError::new(ErrorKind::InvalidData, format!("{}", err)))
    }
}

struct XiWriter(Sender<Value>);

impl Write for XiWriter {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        let value: Value = serde_json::from_slice(buf)?;
        self.0
            .send(value)
            .map(|_| buf.len())
            .map_err(|err| IoError::new(ErrorKind::InvalidData, format!("{}", err)))
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

struct XiReader(Receiver<Value>);

use futures::executor::block_on;

impl Read for XiReader {
    fn read(&mut self, mut buf: &mut [u8]) -> IoResult<usize> {
        let future = async {
            if let Some(value) = self.0.recv().await {
                let data = serde_json::to_string(&value)?;
                buf.write_all(format!("{}\n", data).as_ref())?;
                Ok(data.len() + 1)
            } else {
                Err(IoError::new(
                    ErrorKind::InvalidData,
                    "XiCore Failed to read from channel",
                ))
            }
        };
        block_on(future)
    }
}
