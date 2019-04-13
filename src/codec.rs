use std::io;

use bytes::{BufMut, BytesMut};
use tokio::io::ErrorKind;
use tokio_codec::{Decoder, Encoder, LinesCodec};
use vampirc_uci::{MessageList, parse, Serializable, UciMessage};

pub struct UciCodec {
    delegate: LinesCodec
}

impl UciCodec {
    pub fn new() -> UciCodec {
        UciCodec {
            delegate: LinesCodec::new()
        }
    }
}

impl Encoder for UciCodec {
    type Item = UciMessage;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.delegate.encode(item.serialize(), dst)
    }
}

impl Decoder for UciCodec {
    type Item = UciMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let decode_result = self.delegate.decode(src)?;
        if let Some(dr) = decode_result {
            let ml: MessageList = parse(format!("{}\n", dr).as_str());

            if ml.len() > 1 {
                let err = io::Error::new(ErrorKind::InvalidData, "Too many messages unexpectedly returned by the underlying LinesCodec");
                return Err(err);
            }

            if let Some(m1) = ml.into_iter().next() {
                return Ok(Some(m1));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use vampirc_uci::{UciInfoAttribute, UciMove, UciSquare};

    use super::*;

    #[test]
    fn test_encode() {
        let m: UciMessage = UciMessage::UciOk;

        let mut c = UciCodec::new();
        let mut bm: BytesMut = BytesMut::new();

        c.encode(m, &mut bm).unwrap();

        let s = String::from_utf8(bm.to_ascii_lowercase()).unwrap();
        assert_eq!("uciok\n", s.as_str());
    }

    #[test]
    fn test_encode_complex() {
        let m = UciMessage::Info(vec![
            UciInfoAttribute::from_centipawns(20),
            UciInfoAttribute::Depth(3),
            UciInfoAttribute::Nodes(423),
            UciInfoAttribute::Time(15),
            UciInfoAttribute::Pv(vec![
                UciMove::from_to(UciSquare::from('f', 1), UciSquare::from('c', 4)),
                UciMove::from_to(UciSquare::from('g', 8), UciSquare::from('f', 6)),
                UciMove::from_to(UciSquare::from('b', 1), UciSquare::from('c', 3))
            ])
        ]);

        let mut c = UciCodec::new();
        let mut bm: BytesMut = BytesMut::new();

        c.encode(m, &mut bm).unwrap();

        let s = String::from_utf8(bm.to_ascii_lowercase()).unwrap();
        assert_eq!("info score cp 20 depth 3 nodes 423 time 15 pv f1c4 g8f6 b1c3\n", s.as_str());
    }

    #[test]
    fn test_decode() {
        let m: UciMessage = UciMessage::UciOk;
        let expected_m = m.clone();

        let mut c = UciCodec::new();
        let mut bm: BytesMut = BytesMut::new();

        c.encode(m, &mut bm).unwrap();

        let mr = c.decode(&mut bm).unwrap().unwrap();
        assert_eq!(expected_m, mr);
    }

    #[test]
    fn test_decode_complex() {
        let m = UciMessage::Info(vec![
            UciInfoAttribute::from_centipawns(20),
            UciInfoAttribute::Depth(3),
            UciInfoAttribute::Nodes(423),
            UciInfoAttribute::Time(15),
            UciInfoAttribute::Pv(vec![
                UciMove::from_to(UciSquare::from('f', 1), UciSquare::from('c', 4)),
                UciMove::from_to(UciSquare::from('g', 8), UciSquare::from('f', 6)),
                UciMove::from_to(UciSquare::from('b', 1), UciSquare::from('c', 3))
            ])
        ]);
        let expected_m = m.clone();

        let mut c = UciCodec::new();
        let mut bm: BytesMut = BytesMut::new();

        c.encode(m, &mut bm).unwrap();

        let mr = c.decode(&mut bm).unwrap().unwrap();
        assert_eq!(expected_m, mr);
    }
}