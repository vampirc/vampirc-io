use std::error::Error as StdError;
use std::iter::Map;
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
    mapper: &'a Fn(D::Item) -> Vec<M>,
}

#[cfg(feature = "vampirc-uci")]
type UciParser<'a> = Parser<'a, Stdin, LinesCodec, UciMessage>;

impl<'a, R: AsyncRead, D: Decoder, M> Parser<'a, R, D, M> {
    pub fn new(async_reader: R, decoder: D, mapper: &'a Fn(D::Item) -> Vec<M>) -> Parser<'a, R, D, M> {
        Parser {
            stream: VampircIoStream::<R>(async_reader),
            decoder,
            mapper,
        }
    }

    pub fn poll_msg<F>(self, consumer: F) where F: Fn(M) -> () {
        let f = self.mapper;

        let stream1: FramedRead<R, D> = self.stream.into_frame_stream::<D>(self.decoder);
        let stream2 = stream1.map(|item: D::Item| {
            let v: Vec<M> = f(item);
            let fm: IterOk<IntoIter<M>, D::Error> = stream::iter_ok(v.into_iter());
            fm
        });
        let stream3 = stream2.flatten();

        stream3.and_then(|m: M| {
            consumer(m);
            Ok(())
        }
        )
            .poll();
    }
}


impl<'a, M> Parser<'a, Stdin, LinesCodec, M> {
    pub fn from_stdin(mapper: &'a Fn(String) -> Vec<M>) -> Parser<'a, Stdin, LinesCodec, M> {
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
    parse(s.as_str())
}

