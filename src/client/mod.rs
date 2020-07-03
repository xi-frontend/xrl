mod child;
pub use self::child::ChildProcess;

mod ext;
pub use self::ext::ClientExt;

mod thread;
pub use self::thread::Thread;

use serde_json::Value;

use std::io::Result as IoResult;

use crate::protocol::Message;
use crate::XiLocation;

#[derive(Debug, PartialEq, Clone)]
pub struct ActiveRequest {
    id: usize,
    data: RequestData,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RequestData {
    NewView { file_path: Option<String> },
}

#[async_trait::async_trait]
pub trait ClientImpl: Send {
    fn next_id(&mut self) -> usize;

    async fn receive(&mut self) -> IoResult<Message>;

    async fn send(&mut self, msg: serde_json::Value) -> IoResult<()>;
}

fn get_client_impl(location: XiLocation) -> IoResult<Box<dyn ClientImpl>> {
    match location {
        XiLocation::Embeded => Ok(Box::new(Thread::new()?)),
        XiLocation::Path { cmd } => Ok(Box::new(ChildProcess::new(&cmd)?)),
        XiLocation::File { path } => Ok(Box::new(ChildProcess::new(path.to_str().unwrap())?)),
    }
}

pub struct Client {
    inner: Box<dyn ClientImpl>,
}

impl Client {
    pub fn new(xi: XiLocation) -> IoResult<Client> {
        Ok(Client {
            inner: get_client_impl(xi)?,
        })
    }
}

#[async_trait::async_trait]
impl ClientImpl for Client {
    fn next_id(&mut self) -> usize {
        self.inner.next_id()
    }

    async fn receive(&mut self) -> IoResult<Message> {
        self.inner.receive().await
    }

    async fn send(&mut self, msg: Value) -> IoResult<()> {
        self.inner.send(msg).await
    }
}
