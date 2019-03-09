use std::vec::IntoIter;

use tokio::codec::*;
use tokio::io::*;
use tokio::prelude::*;
use tokio::prelude::stream::*;
#[cfg(feature = "vampirc-uci")]
use vampirc_uci::{parse, UciMessage};

use crate::io::VampircIoStream;

pub struct Parser<'a, R: AsyncRead, D: Decoder, M: Sized> {
    stream: VampircIoStream<R>,
    decoder: D,
    mapper: &'a (FnMut(D::Item) -> Vec<M> + 'a),
}

#[cfg(feature = "vampirc-uci")]
type UciParser<'a> = Parser<'a, Stdin, LinesCodec, UciMessage>;

impl<'a, R: AsyncRead, D: Decoder, M> Parser<'a, R, D, M> {
    pub fn new(async_reader: R, decoder: D, mapper: &'a FnMut(D::Item) -> Vec<M>) -> Parser<'a, R, D, M> {
        Parser {
            stream: VampircIoStream::<R>(async_reader),
            decoder,
            mapper,
        }
    }

    pub fn poll_msg<F>(self, mut consumer: F) where F: FnMut(M) -> () {
        let f: &'a mut (FnMut(D::Item) -> Vec<M> + 'a) = self.mapper;

        let stream1: FramedRead<R, D> = self.stream.into_frame_stream::<D>(self.decoder);
        let stream2 = stream1.map(|item: D::Item| {
            let v: Vec<M> = f(item);
            let fm: IterOk<IntoIter<M>, D::Error> = stream::iter_ok(v.into_iter());
            fm
        });
        let stream3 = stream2.flatten();

        let mut stream4 = stream3.and_then(|m: M| {
            consumer(m);
            Ok(())
        }
        );

        loop {
            let p = stream4.poll();

            if let Ok(a) = p {
                match a {
                    Async::Ready(something) => {
                        if something.is_none() {
                            break;
                        }
                    },
                    Async::NotReady => continue
                }
            } else {
                break;
            }
        }
    }
}


impl<'a, M> Parser<'a, Stdin, LinesCodec, M> {
    pub fn from_stdin(mapper: &'a FnMut(String) -> Vec<M>) -> Parser<'a, Stdin, LinesCodec, M> {
        Parser::new(stdin(), LinesCodec::new(), mapper)
    }
}

#[cfg(feature = "vampirc-uci")]
impl<'a> UciParser<'a> {
    pub fn from_uci() -> UciParser<'a> {
        Parser::from_stdin(&parse_uci)
    }
}

#[cfg(feature = "vampirc-uci")]
fn parse_uci(s: String) -> Vec<UciMessage> {
    parse((s + "\n").as_str())
}

#[cfg(test)]
mod tests {
    use vampirc_uci::uci::Serializable;

    use crate::io::tests::StringAsyncReader;

    use super::*;

    #[cfg(feature = "vampirc-uci")]
    type TestUciParser<'a> = Parser<'a, StringAsyncReader, LinesCodec, UciMessage>;

    #[test]
    #[cfg(feature = "vampirc-uci")]
    fn test_read_uci() {
        let strings = vec!["uci".to_string(), "go ponder".to_string()];
        let reader = StringAsyncReader::new(strings);

        let pu = &mut parse_uci;
        let tup: TestUciParser = Parser::new(reader, LinesCodec::new(), pu);

        let mut msg: Vec<UciMessage> = vec![];

        tup.poll_msg(|m: UciMessage| {
            println!("{}", m.serialize());
            msg.push(m);
        })
    }
}

