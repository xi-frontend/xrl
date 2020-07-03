use log::trace;
use serde_json::{to_string, Value};
use tokio::io::BufReader;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout, Command};

use std::io::Error as IoError;
use std::io::ErrorKind;
use std::io::Result as IoResult;
use std::process::Stdio;

use crate::client::ClientImpl;
use crate::protocol::Message;

pub struct ChildProcess {
    request_id: usize,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    stderr: BufReader<ChildStderr>,
}

impl ChildProcess {
    pub fn new(cmd: &str) -> IoResult<ChildProcess> {
        let mut inner = Command::new(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("XI_LOG", "trace")
            .spawn()?;
        let stdin = inner
            .stdin
            .take()
            .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "failed to take child input"))?;
        let stdout =
            BufReader::new(inner.stdout.take().ok_or_else(|| {
                IoError::new(ErrorKind::InvalidData, "Failed to read child output")
            })?);
        let stderr =
            BufReader::new(inner.stderr.take().ok_or_else(|| {
                IoError::new(ErrorKind::InvalidData, "Failed to read child output")
            })?);
        Ok(ChildProcess {
            request_id: 0,
            stdin,
            stdout,
            stderr,
        })
    }
}

#[async_trait::async_trait]
impl ClientImpl for ChildProcess {
    fn next_id(&mut self) -> usize {
        self.request_id += 1;
        self.request_id - 1
    }

    async fn receive(&mut self) -> IoResult<Message> {
        let stdout = &mut self.stdout;
        let stderr = &mut self.stderr;
        let mut stderr_line = String::new();
        let mut stdout_line = String::new();
        tokio::select! {
            Ok(_) = stdout.read_line(&mut stdout_line) => {
                trace!("client < xi-core: {}", stdout_line);
                Ok(serde_json::from_slice::<Message>(stdout_line.as_bytes()).unwrap())
            }
            Ok(_) = stderr.read_line(&mut stderr_line) => {
                trace!("client < xi-core: {}", stderr_line);
                Ok(Message::Error(stderr_line))
            }
        }
    }

    async fn send(&mut self, msg: Value) -> IoResult<()> {
        let data = format!("{}\n", to_string(&msg)?);
        trace!("client > xi-core: {}", data);
        self.stdin.write_all(data.as_ref()).await?;
        Ok(())
    }
}
