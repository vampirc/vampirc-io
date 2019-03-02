use std::io::Error;

use tokio::codec::*;
use tokio::io::*;
use tokio::prelude::*;

pub struct VampircIoStream<R>(R) where R: AsyncRead;


impl<R> Stream for VampircIoStream<R> where R: AsyncRead {
    type Item = [u8; 64];
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<[u8; 64]>>> {
        let mut buf: [u8; 64] = [0; 64];

        match self.0.poll_read(&mut buf) {
            Ok(Async::Ready(n)) => {
                // By convention, if an AsyncRead says that it read 0 bytes,
                // we should assume that it has got to the end, so we signal that
                // the Stream is done in this case by returning None:
                if n == 0 {
                    Ok(Async::Ready(None))
                } else {
                    Ok(Async::Ready(Some(buf)))
                }
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e)
        }
    }
}

impl<R> VampircIoStream<R> where R: AsyncRead {
    pub fn into_frame_stream<D>(self, decoder: D) -> FramedRead<R, D> where D: Decoder {
        FramedRead::new(self.0, decoder)
    }

    pub fn into_line_stream(self) -> FramedRead<R, LinesCodec> {
        self.into_frame_stream(LinesCodec::new())
    }
}

impl VampircIoStream<Stdin> {
    pub fn stdin() -> VampircIoStream<Stdin> {
        VampircIoStream(stdin())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    struct StringAsyncReader {
        pub lines: Vec<String>,
        full_str: Vec<u8>,
        location: usize,
    }

    impl StringAsyncReader {
        pub fn new(lines: Vec<String>) -> StringAsyncReader {
            let mut bv: Vec<u8> = Vec::new();

            for l in lines.iter() {
                let ln = l.clone() + "\n";

                for b in ln.as_bytes().iter() {
                    bv.push(*b);
                }
            }

            StringAsyncReader {
                lines,
                full_str: bv,
                location: 0,
            }
        }
    }

    impl Read for StringAsyncReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let bl = buf.len();
            let remaining = self.full_str.len() - self.location;

            let to_read = bl.min(remaining);

            if to_read == 0 {
                return Ok(0);
            }

            let bytes = &self.full_str[self.location..to_read];

            for (i, b) in bytes.into_iter().enumerate() {
                buf[i] = *b;
            }

            self.location += to_read;

            Ok(to_read)
        }
    }

    impl AsyncRead for StringAsyncReader {}


    #[test]
    fn test_read_lines() {
        let strings = vec!["uci".to_string(), "go ponder".to_string()];
        let reader = StringAsyncReader::new(strings);
        let stream = VampircIoStream(reader);


        stream.into_line_stream()
            .for_each(|line| {
                println!("{}", line);
                Ok(())
            }).poll().expect("Polling failed");
    }
}