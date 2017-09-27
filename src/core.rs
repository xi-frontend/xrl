use futures::Poll;
use protocol::{Endpoint, Service};
use client::Client;
use std::io::{self, Read, Write};
use std::process::Command;
use std::process::Stdio;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_process::{Child, ChildStdin, ChildStdout, CommandExt};
use tokio_core::reactor::Handle;

struct Core {
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

pub fn spawn<S: Service>(executable: &str, server: S, handle: &Handle) -> Client {
    let mut xi_core = Command::new(executable)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .env("RUST_BACKTRACE", "1")
        .spawn_async(handle)
        .expect("failed to spawn xi-core");

    let stdout = xi_core.stdout().take().unwrap();
    let stdin = xi_core.stdin().take().unwrap();
    let core = Core {
        core: xi_core,
        stdout: stdout,
        stdin: stdin,
    };
    let mut endpoint = Endpoint::new(core);
    endpoint.set_server(server);
    Client(endpoint.set_client())
}
