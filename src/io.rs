use std::io;

use tokio::io::{shutdown, Stdin, stdin, Stdout, stdout};
use tokio::prelude::{Async, AsyncRead, AsyncWrite, Future, Read, Stream};
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

#[cfg(test)]
mod tests {
    use tokio::codec::LinesCodec;
    use tokio_codec::FramedRead;

    use super::*;

//    #[test]
//    fn test_message_read_output() {
//
//
//
//        let mut ios = new_uci_engine_stream();
//        ios.
//
//        let f = ios.for_each(|l| {
//            println!("Message: {}", l);
//            Ok(())
//        });
//
//        let p = f.and_then(|a| {
//            shutdown()
//                .map(drop)
//                .map_err(drop)
//        });
//
//        tokio::run(p);
//
//    }

    #[test]
    fn test_interactive_stdin_read_async() {
        print!("Input >>> ");

        let frs = FramedRead::new(stdin(), LinesCodec::new());
        let proc = frs.for_each(|m| {
            println!("Message: {}", m);
            Ok(())
        })
            .map_err(|e| {
                println!("ERROR: {}", e);
            })

            ;

        tokio::run(proc);
    }
}

