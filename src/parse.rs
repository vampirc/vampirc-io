use std::iter::Map;

use tokio::codec::*;
use tokio::io::*;
use tokio::prelude::*;
use tokio::prelude::stream::Map as StreamMap;

use crate::io::VampircIoStream;

pub struct Parser<'a, R: AsyncRead, D: Decoder, M: Sized> {
    stream: VampircIoStream<R>,
    decoder: D,
    mapper: &'a Fn(D::Item) -> M,
}

impl<'a, R: AsyncRead, D: Decoder, M> Parser<'a, R, D, M> {
    pub fn new(async_reader: R, decoder: D, mapper: &'a Fn(D::Item) -> M) -> Parser<'a, R, D, M> {
        Parser {
            stream: VampircIoStream(async_reader),
            decoder,
            mapper,
        }
    }

    pub fn poll_msg<F>(self, consumer: F) where F: Fn(M) -> () {
        let f = self.mapper;

        let stream = self.stream.into_frame_stream(self.decoder)
            .map(|item| f(item))
            .and_then(|m: M| {
                consumer(m);
                Ok(())
            })


            ;
    }
}

impl<'a, M> Parser<'a, Stdin, LinesCodec, M> {
    pub fn from_stdin(mapper: &'a Fn(String) -> M) -> Parser<'a, Stdin, LinesCodec, M> {
        Parser::new(stdin(), LinesCodec::new(), mapper)
    }
}