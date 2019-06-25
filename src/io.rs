use std::fmt::Error;
use std::io;

use crossbeam::channel::unbounded;
use crossbeam::queue::ArrayQueue;
use futures::lazy;
use tokio::io::{shutdown, stdin, stdout, ErrorKind, Stdin, Stdout};
use tokio::prelude::{Async, AsyncRead, AsyncWrite, Future, Read, Sink, Stream};
use tokio_codec::{Decoder, Framed};
use vampirc_uci::{CommunicationDirection, MessageList, UciMessage};

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

pub fn run_forever<S, H, E>(
    frame: UciStream<S>,
    msg_reader: H,
    inbound: Box<Stream<Item = UciMessage, Error = io::Error> + Send + Sync>,
    msg_handler: H,
    err_handler: E,
    outbound: Box<Stream<Item = UciMessage, Error = io::Error> + Send + Sync>,
) where
    S: AsyncRead + AsyncWrite + Send + Sync + 'static,
    H: Fn(UciMessage) + Send + Sync + 'static,
    E: Fn(&io::Error, CommunicationDirection) + Send + Sync + Copy + 'static,
{
    tokio::run(lazy(move || {
        let (sink, stream) = frame.split();

        let proc_read = stream
            .for_each(move |m: UciMessage| {
                (msg_reader)(m);
                Ok(())
            })
            .map_err(move |e| {
                (err_handler)(&e, CommunicationDirection::GuiToEngine);
            });

        let proc_handle = inbound
            .for_each(move |m: UciMessage| {
                (msg_handler)(m);
                Ok(())
            })
            .map_err(move |e| {
                (err_handler)(&e, CommunicationDirection::GuiToEngine);
            });

        let proc_write = lazy(move || {
            sink.send_all(outbound);
            Ok(())
        })
        .map_err(move |e| {
            (err_handler)(&e, CommunicationDirection::EngineToGui);
        });

        tokio::spawn(proc_read);
        tokio::spawn(proc_handle);
        tokio::spawn(proc_write);
        Ok(())
    }));
}

pub fn run_f<S, H>(frame: UciStream<S>, msg_reader: H)
where
    S: AsyncRead + AsyncWrite + Send + Sync + 'static,
    H: Fn(UciMessage) + Send + Sync + 'static,
{
}

pub fn run_default() {
    let frame = new_uci_engine_stream();
    let (inb_s, inb_r) = unbounded();
    let (out_s, out_r) = unbounded();

    let (sink, stream) = frame.split();

    let proc_read = stream
        .for_each(move |m: UciMessage| {
            let r = inb_s.try_send(m);
            if r.is_err() {
                return Err(io::Error::new(ErrorKind::WouldBlock, r.err().unwrap()));
            }
            Ok(())
        })
        .map_err(move |e| {
            println!("{}", e);
        });

    let proc_handle = lazy(move || {
        loop {
            if let Ok(m) = inb_r.try_recv() {
                println!("Received msg: {}", m);
            }
        }
        Ok(())
    })
    .map_err(move |e: io::Error| {
        println!("{}", e);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
}
