use std::vec::IntoIter;

use tokio::codec::*;
use tokio::io::*;
use tokio::prelude::*;
use tokio::prelude::stream::*;
#[cfg(feature = "vampirc-uci")]
use vampirc_uci::{parse, UciMessage};

use crate::io::VampircIoStream;

pub struct Parser<R: AsyncRead, D: Decoder, M: Sized> {
    stream: VampircIoStream<R>,
    decoder: D,
    mapper: fn(D::Item) -> Vec<M>
}

type StringParser<R> = Parser<R, LinesCodec, String>;
type StdinStringParser<> = StringParser<Stdin>;

#[cfg(feature = "vampirc-uci")]
type UciParser<> = Parser<Stdin, LinesCodec, UciMessage>;

impl<R: AsyncRead, D: Decoder, M> Parser<R, D, M> {
    pub fn new(async_reader: R, decoder: D, mapper: fn(D::Item) -> Vec<M>) -> Parser<R, D, M> {
        Parser {
            stream: VampircIoStream::<R>(async_reader),
            decoder,
            mapper,
        }
    }

    pub fn poll_msg<F>(self, mut consumer: F) where F: FnMut(M) -> () {
        let f: fn(D::Item) -> Vec<M> = self.mapper;

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


impl<M: Sized> Parser<Stdin, LinesCodec, M> {
    pub fn from_stdin(mapper: fn(String) -> Vec<M>) -> Parser<Stdin, LinesCodec, M> {
        Parser::new(stdin(), LinesCodec::new(), mapper)
    }
}

#[cfg(feature = "vampirc-uci")]
impl<> UciParser<> {
    pub fn new_uci() -> UciParser<> {
        Parser::from_stdin(UciParser::parse_uci)
    }

    fn parse_uci(s: String) -> Vec<UciMessage> {
        parse((s + "\n").as_str())
    }
}


impl<R: AsyncRead> StringParser<R> {
    pub fn new_string(async_reader: R) -> StringParser<R> {
        return Parser::new(async_reader, LinesCodec::new(), StringParser::<R>::parse_str)
    }

    fn parse_str(s: String) -> Vec<String> {
        let v: Vec<String> = vec![s];

        v
    }
}

impl StdinStringParser {
    pub fn new_stdin_string() -> StdinStringParser {
        StringParser::<Stdin>::new_string(stdin())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use vampirc_uci::uci::{Serializable, UciTimeControl};

    use crate::io::tests::StringAsyncReader;

    use super::*;

    #[cfg(feature = "vampirc-uci")]
    type TestUciParser<> = Parser<StringAsyncReader, LinesCodec, UciMessage>;

    type CustomParser<> = Parser<StringAsyncReader, BytesCodec, String>;

    #[test]
    #[cfg(feature = "vampirc-uci")]
    fn test_read_uci() {
        let strings = vec!["uci".to_string(), "go ponder".to_string()];
        let reader = StringAsyncReader::new(strings);

        let tup: TestUciParser = Parser::new(reader, LinesCodec::new(), UciParser::parse_uci);

        let mut msg: Vec<UciMessage> = vec![];

        tup.poll_msg(|m: UciMessage| {
            println!("{}", m.serialize());
            msg.push(m);
        });

        assert_eq!(msg.len(), 2);
        assert_eq!(msg[0], UciMessage::Uci);
        assert_eq!(msg[1], UciMessage::Go {
            time_control: Some(UciTimeControl::Ponder),
            search_control: None,
        });
    }

    #[test]
    fn test_custom_parser() {
        let strings = vec!["abc".to_string(), "def".to_string()];
        let reader = StringAsyncReader::new(strings);

        let mut m: Vec<String> = vec![];
        let cp: CustomParser = Parser::new(reader, BytesCodec::new(), parse_custom);

        cp.poll_msg(|s: String| {
            println!("{}", s);
            m.push(s);
        });

        assert_eq!(m.len(), 1);

        assert_eq!(m[0].as_str(), "ABC\nDEF\n");
    }

    fn parse_custom(b: BytesMut) -> Vec<String> {
        let s = String::from_utf8(b.to_ascii_uppercase()).unwrap();
        let v: Vec<String> = vec![s];

        v
    }

    #[test]
    fn test_string_parser() {
        let strings = vec!["AbC".to_string(), "DeF".to_string()];
        let reader = StringAsyncReader::new(strings);

        let mut m: Vec<String> = vec![];
        let cp: StringParser<StringAsyncReader> = StringParser::new_string(reader);

        cp.poll_msg(|s: String| {
            println!("{}", s);
            m.push(s);
        });

        assert_eq!(m.len(), 2);

        assert_eq!(m[0].as_str(), "AbC");
        assert_eq!(m[1].as_str(), "DeF");
    }
}

