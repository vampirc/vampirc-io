use tokio::codec::Encoder;
use tokio::prelude::AsyncWrite;

pub struct Serializer<W: AsyncWrite, E: Encoder, M: Sized> {
    write: W,
    encoder: E,
    mapper: fn(M) -> [u8],
}