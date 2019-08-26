use std::fmt::Error;
use std::io;
//use crossbeam::channel::{unbounded, Sender, Receiver};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crossbeam::queue::ArrayQueue;
use futures::lazy;
use tokio::io::{ErrorKind, shutdown, stdin, Stdin, stdout, Stdout};
use tokio::prelude::{Async, AsyncRead, AsyncWrite, Future, Read, Sink, Stream};
use tokio_codec::{Decoder, Framed};
use vampirc_uci::{CommunicationDirection, MessageList, UciMessage};

use crate::codec::UciCodec;

pub type UciStream<S: AsyncRead + AsyncWrite + Sized> = Framed<S, UciCodec>;
pub type UciEngineStream = UciStream<StdinStdout>;
pub type MessageChannel = (Sender<UciMessage>, Receiver<UciMessage>);

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

pub fn run_engine<H, E>(mut msg_handler: H, mut err_handler: E)
where
    for<'a> H: FnMut(&'a UciMessage) + Send + Sync + 'static,
    for<'a> E: FnMut(&'a io::Error) + Send + 'static,
{
    run(new_uci_engine_stream(), msg_handler, err_handler);
}

pub fn run<S, H, E>(stream: UciStream<S>, mut msg_handler: H, mut err_handler: E)
where
    S: AsyncRead + AsyncWrite + Sized + Send + Sync + 'static,
    for<'a> H: FnMut(&'a UciMessage) + Send + 'static,
    for<'a> E: FnMut(&'a io::Error) + Send + 'static,
{
    let proc = stream
        .for_each(move |m: UciMessage| {
            msg_handler(&m);
            Ok(())
        })
        .map_err(move |e| {
            err_handler(&e);
        });

    tokio::run(proc);
}

pub fn run_c<S>(frame: UciStream<S>, msg_src: &'static MessageChannel)
    where S: AsyncRead + AsyncWrite + Send + Sync + 'static {
    let (_, src) = msg_src;
    let src_clone = src.clone();
    let (sink, stream) = frame.split();

    thread::spawn(move || {

        loop {
            let m = src.recv().unwrap();
            sink.send(m);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
}
