use std::io;

use tokio::io::{Stdin, stdin, Stdout, stdout};
use tokio::prelude::{Async, AsyncRead, AsyncWrite, Read};
use tokio_codec::{Decoder, Framed};

use crate::codec::UciCodec;

pub type UciStream<S> = Framed<S, UciCodec>;
pub type UciEngineStream = UciStream<StdinStdout>;

#[derive(Debug)]
pub struct StdinStdout {
    pub stdin: Stdin,
    pub stdout: Stdout,
}


pub fn stdin_stdout() -> StdinStdout {
    StdinStdout::new()
}

impl StdinStdout {
    fn new() -> StdinStdout {
        StdinStdout {
            stdin: stdin(),
            stdout: stdout(),
        }
    }
}

impl io::Read for StdinStdout {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.stdin.read(buf)
    }
}

impl AsyncRead for StdinStdout {
    unsafe fn prepare_uninitialized_buffer(&self, b: &mut [u8]) -> bool {
        self.stdin.prepare_uninitialized_buffer(b)
    }
}

impl io::Write for StdinStdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.stdout.flush()
    }
}

impl AsyncWrite for StdinStdout {
    fn shutdown(&mut self) -> Result<Async<()>, io::Error> {
        self.stdout.shutdown()
    }
}

pub fn new_uci_stream<S: AsyncRead + AsyncWrite>(stream: S) -> UciStream<S> {
    UciCodec::new().framed(stream)
}

pub fn new_uci_engine_stream() -> UciEngineStream {
    new_uci_stream(stdin_stdout())
}

