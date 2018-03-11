use futures::{Future, Poll, Stream};
use bytes::BytesMut;
use protocol::Endpoint;
use client::Client;
use std::io::{self, Read, Write};
use std::process::Command;
use std::process::Stdio;
use tokio_io::{codec, AsyncRead, AsyncWrite};
use tokio_process::{Child, ChildStderr, ChildStdin, ChildStdout, CommandExt};
use tokio_core::reactor::Handle;
use frontend::{Frontend, FrontendBuilder};
use std::clone::Clone;

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

/// Start Xi core, and return a client and a stream of Xi's stderr lines.
pub fn spawn<B, F>(executable: &str, builder: B, handle: &Handle) -> (Client, CoreStderr)
where
    B: FrontendBuilder<F> + 'static,
    F: Frontend + 'static,
{
    let mut xi_core = Command::new(executable)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .env("RUST_BACKTRACE", "1")
        .spawn_async(handle)
        .expect("failed to spawn xi-core");

    let stdout = xi_core.stdout().take().unwrap();
    let stdin = xi_core.stdin().take().unwrap();
    let stderr = xi_core.stderr().take().unwrap();
    let core = Core {
        core: xi_core,
        stdout,
        stdin,
    };

    let mut endpoint = Endpoint::new(core);
    let client = Client(endpoint.set_client());
    let service = builder.build(client.clone());
    endpoint.set_server(service);
    handle.spawn(endpoint.map_err(|_| ()));
    (client, CoreStderr::new(stderr))
}

struct LineCodec;

// straight from
// https://github.com/tokio-rs/tokio-line/blob/master/simple/src/lib.rs
impl codec::Decoder for LineCodec {
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
pub struct CoreStderr(codec::FramedRead<ChildStderr, LineCodec>);

impl CoreStderr {
    fn new(stderr: ChildStderr) -> Self {
        CoreStderr(codec::FramedRead::new(stderr, LineCodec {}))
    }
}

impl Stream for CoreStderr {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll()
    }
}
