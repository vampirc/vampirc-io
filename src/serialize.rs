use tokio::codec::{Encoder, LinesCodec};
use tokio::io::Stdout;
use tokio::prelude::AsyncWrite;
#[cfg(feature = "vampirc-uci")]
use vampirc_uci::UciMessage;

pub struct Serializer<W: AsyncWrite, E: Encoder, M: Sized> {
    write: W,
    encoder: E,
    mapper: fn([u8]) -> M,
}

pub type StringSerializer<R> = Serializer<R, LinesCodec, String>;
pub type StdoutStringSerializer<> = StringSerializer<Stdout>;

#[cfg(feature = "vampirc-uci")]
pub type UciSerializer<> = Serializer<Stdout, LinesCodec, UciMessage>;