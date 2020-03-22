use crate::client::Client;
use crate::frontend::{Frontend, FrontendBuilder};
use crate::protocol::Endpoint;
use crate::ClientError;
use bytes::BytesMut;
use futures::{Future, Poll, Stream};
use std::io::{self, Read, Write};
use std::process::Command;
use std::process::Stdio;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_codec::{Decoder, FramedRead};
use tokio_process::{Child, ChildStderr, ChildStdin, ChildStdout, CommandExt};

struct Core {
    #[allow(dead_code)]
    core: Child,
    stdout: ChildStdout,
    stdin: ChildStdin,
}

impl Read for Core {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdout.read(buf)
    }
}

impl AsyncRead for Core {
    // FIXME: do I actually have to implement this?
    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [u8]) -> bool {
        self.stdout.prepare_uninitialized_buffer(buf)
    }
}

impl Write for Core {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdin.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdin.flush()
    }
}

impl AsyncWrite for Core {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        self.stdin.shutdown()
    }
}

/// Start Xi core, and spawn an RPC client on the current tokio executor.
///
/// # Panics
///
/// This function calls
/// [`tokio::spawn`](https://docs.rs/tokio/0.1.21/tokio/executor/fn.spawn.html)
/// so it will panic if the default executor is not set or if spawning
/// onto the default executor returns an error.
pub fn spawn<B, F>(executable: &str, builder: B) -> Result<(Client, CoreStderr), ClientError>
where
    F: Frontend + 'static + Send,
    B: FrontendBuilder<Frontend = F> + 'static,
{
    spawn_command(Command::new(executable), builder)
}

/// Same as [`spawn`] but accepts an arbitrary [`std::process::Command`].
pub fn spawn_command<B, F>(
    mut command: Command,
    builder: B,
) -> Result<(Client, CoreStderr), ClientError>
where
    F: Frontend + 'static + Send,
    B: FrontendBuilder<Frontend = F> + 'static,
{
    info!("starting xi-core");
    let mut xi_core = command
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .env("RUST_BACKTRACE", "1")
        .spawn_async()?;

    let stdout = xi_core.stdout().take().unwrap();
    let stdin = xi_core.stdin().take().unwrap();
    let stderr = xi_core.stderr().take().unwrap();
    let core = Core {
        core: xi_core,
        stdout,
        stdin,
    };

    let (endpoint, client) = Endpoint::new(core, builder);

    info!("spawning the Xi-RPC endpoint");
    // XXX: THIS PANICS IF THE DEFAULT EXECUTOR IS NOT SET
    tokio::spawn(endpoint.map_err(|e| error!("Endpoint exited with an error: {:?}", e)));

    Ok((Client(client), CoreStderr::new(stderr)))
}

pub struct LineCodec;

// straight from
// https://github.com/tokio-rs/tokio-line/blob/master/simple/src/lib.rs
impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<String>, io::Error> {
        if let Some(n) = buf.as_ref().iter().position(|b| *b == b'\n') {
            let line = buf.split_to(n);
            buf.split_to(1);
            return match ::std::str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid string")),
            };
        }
        Ok(None)
    }
}

/// A stream of Xi core stderr lines
pub struct CoreStderr(FramedRead<ChildStderr, LineCodec>);

impl CoreStderr {
    fn new(stderr: ChildStderr) -> Self {
        CoreStderr(FramedRead::new(stderr, LineCodec {}))
    }
}

impl Stream for CoreStderr {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll()
    }
}
