use std::vec::IntoIter;

use tokio::codec::*;
use tokio::io::*;
use tokio::prelude::*;
use tokio::prelude::stream::*;
#[cfg(feature = "vampirc-uci")]
use vampirc_uci::{parse, UciMessage};

pub struct Serializer<W: AsyncWrite, E: Encoder, M: Sized> {
    write: W,
    encoder: E,
    mapper: fn(&M) -> E::Item,
}

pub type StringSerializer<R> = Serializer<R, LinesCodec, String>;
pub type StdoutStringSerializer<> = StringSerializer<Stdout>;

#[cfg(feature = "vampirc-uci")]
pub type UciSerializer<> = Serializer<Stdout, LinesCodec, UciMessage>;

impl<W: AsyncWrite, E: Encoder, M: Sized> Serializer<W, E, M> {

//    pub fn push_msg(self, messages: &Vec<M>) {
//        let fw = FramedWrite::new(self.write, self.encoder);
//
//        // let fm: IterOk<IntoIter<M>, D::Error> = stream::iter_ok(v.into_iter());
//        let fm: IterOk<IntoIter<M>, E::Error> = stream::iter_ok(messages.into_iter());
//
//        fm
//            .map(|m| (self.mapper)(&m))
//            .for_each(|item| {
//                fw.send(item);
//            });
//
//
//
//    }
}