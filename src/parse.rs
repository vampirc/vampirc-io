use tokio::codec::*;
use tokio::io::*;
use tokio::prelude::*;
use tokio::prelude::stream::Map as StreamMap;

use crate::io::VampircIoStream;

pub struct Parser<'a, R: AsyncRead, D: Decoder, M> {
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

//    pub fn parse_msg(self)  {
//        return self.stream.into_frame_stream(self.decoder)
//            .map( |v| self.mapper.call(v))
//
//    }
}

impl<'a, M> Parser<'a, Stdin, LinesCodec, M> {
    pub fn from_stdin(mapper: &'a Fn(String) -> M) -> Parser<'a, Stdin, LinesCodec, M> {
        Parser::new(stdin(), LinesCodec::new(), mapper)
    }
}