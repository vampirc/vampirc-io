use std::io;

use tokio::io::{shutdown, stdin, Stdin, stdout, Stdout};
use tokio::prelude::{Async, AsyncRead, AsyncWrite, Future, Read, Stream};
use tokio_codec::{Decoder, Framed};
use vampirc_uci::{MessageList, UciMessage};

use crate::codec::UciCodec;

pub type UciStream<S: AsyncRead + AsyncWrite + Sized> = Framed<S, UciCodec>;
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

pub fn new_uci_stream<S: AsyncRead + AsyncWrite + Sized>(stream: S) -> UciStream<S> {
    UciCodec::new().framed(stream)
}

pub fn new_uci_engine_stream() -> UciEngineStream {
    new_uci_stream(stdin_stdout())
}

pub fn run_engine<H, E>(stream: &'static mut UciEngineStream, mut msg_handler: H, mut err_handler: E)
    where
        H: FnMut(&UciMessage, &'static mut UciEngineStream) + Send + Sync + 'static,
        E: FnMut(&io::Error) + Send + 'static
{
    run(stream, msg_handler, err_handler);
}

pub fn run<S, H, E>(stream: &'static mut UciStream<S>, mut msg_handler: H, mut err_handler: E)
    where
        S: AsyncRead + AsyncWrite + Sized + Send + Sync + 'static,
        H: FnMut(&UciMessage, &'static mut UciStream<S>) + Send + 'static,
        E: FnMut(&io::Error) + Send + 'static
{
    let proc = stream
        .for_each(move |m: UciMessage| {
            msg_handler(&m, stream);
            Ok(())
        })
        .map_err(move |e| {
            err_handler(&e);
        });

    tokio::run(proc);
}


#[cfg(test)]
mod tests {
    use super::*;
}
