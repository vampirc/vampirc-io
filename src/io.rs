#![feature(async_await)]

use std::error::Error;
use std::io;

use futures::{AsyncWriteExt, executor, Sink, Stream};
use futures::io::{AllowStdIo, AsyncReadExt, IntoSink};
use vampirc_uci::{ByteVecUciMessage, UciMessage};

async fn async_write() {
    let mut stdout = AllowStdIo::new(io::stdout());
    stdout.write_all("Whatever\n".as_bytes()).await;
}

pub struct MessageSinkStream<I: Stream<Item=UciMessage>, O: Sink<ByteVecUciMessage>> {
    inbound: I,
    outbound: O,
}

impl<I> MessageSinkStream<I, IntoSink<AllowStdIo<io::Stdout>, ByteVecUciMessage>> where I: Stream<Item=UciMessage> {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_async_write() {
        executor::block_on(async {
            async_write().await;
        });
    }
}