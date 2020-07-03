use crate::client::{Client, ClientExt, ClientImpl};
use crate::protocol::Message;
use crate::XiLocation;

use serde_json::Value;
use tokio::time::timeout;

use std::io;
use std::path::Path;
use std::time::Duration;

/// Special client with extra methods to make testing easier.
pub struct TestClient {
    fail_on_errors: bool,
    inner: Client,
}

#[allow(dead_code)]
impl TestClient {
    /// Create a new TestClient.
    pub fn new(location: XiLocation) -> io::Result<TestClient> {
        Ok(TestClient {
            inner: Client::new(location)?,
            fail_on_errors: true,
        })
    }

    pub fn fail_on_error(&mut self, b: bool) {
        self.fail_on_errors = b;
    }

    /// Creates a new client and sends the `client_started` notification to xi-core.
    pub async fn from_location(location: XiLocation) -> io::Result<TestClient> {
        let mut inner = Client::new(location)?;
        inner.client_started(None, None).await?;
        Ok(TestClient {
            inner,
            fail_on_errors: true,
        })
    }

    /// Helper function that will create a TestClient using the embeded xi-core
    /// using the from_location function.
    pub async fn embeded() -> io::Result<TestClient> {
        TestClient::from_location(XiLocation::Embeded).await
    }
    /// Helper function that will create a TestClient using the specified `path` xi-core
    /// using the from_location function.
    pub async fn file<F: AsRef<Path>>(path: F) -> io::Result<TestClient> {
        let location = XiLocation::File {
            path: path.as_ref().to_path_buf(),
        };
        TestClient::from_location(location).await
    }

    /// Helper function that will create a TestClient using the specified `cmd` xi-core
    /// using the from_location function.
    pub async fn path<S: Into<String>>(cmd: S) -> io::Result<TestClient> {
        let location = XiLocation::Path { cmd: cmd.into() };
        TestClient::from_location(location).await
    }

    /// Function to check if a particular Message is received from XiCore.
    /// `max_reqs` is the maximum number of requests to read from xi-core, will stop
    /// and fail if the number of requests exceeds `max_reqs`.
    /// Will only try to read from xi-core for 5 secs and then will fail if no data was received.
    pub async fn check_responses(
        &mut self,
        max_reqs: Option<usize>,
        expected: Message,
    ) -> io::Result<()> {
        let max_reqs = max_reqs.unwrap_or(5);
        let mut try_counter: usize = 1;
        loop {
            if try_counter == max_reqs && self.fail_on_errors {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Xi didnt send the expected notification",
                ));
            }
            let msg = timeout(Duration::from_secs(5), self.inner.get()).await??;
            if let Message::Error(err) = &msg {
                if self.fail_on_errors {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Xi sent an error: {}", err),
                    ));
                }
            }
            if msg == expected {
                break;
            }
            try_counter += 1;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ClientImpl for TestClient {
    fn next_id(&mut self) -> usize {
        self.inner.next_id()
    }

    async fn receive(&mut self) -> io::Result<Message> {
        self.inner.receive().await
    }

    async fn send(&mut self, msg: Value) -> io::Result<()> {
        self.inner.send(msg).await
    }
}
