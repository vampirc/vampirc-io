use std::convert::TryInto;
use std::ops::DerefMut;
use std::sync::Arc;

use async_std::future::ready;
use async_std::io;
use async_std::io::{BufRead, ErrorKind};
use async_std::sync::{Mutex, RwLock};
use async_std::task::block_on;
use futures::{AsyncRead, AsyncWriteExt, future, FutureExt, join, Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::sink::{Buffer, With};
use vampirc_uci::{ByteVecUciMessage, MessageList, parse_strict, parse_with_unknown, Serializable, UciMessage};

pub type UciStream = dyn Stream<Item=Result<UciMessage, io::Error>> + Unpin + Send + Sync;
pub type UciSink = dyn Sink<UciMessage, Error=io::Error> + Unpin + Send;
pub type UciSender = UnboundedSender<UciMessage>;
pub type UciReceiver = UnboundedReceiver<UciMessage>;
pub type UciTrySender = UnboundedSender<io::Result<UciMessage>>;
pub type UciTryReceiver = UnboundedReceiver<io::Result<UciMessage>>;



pub fn from_reader<'a, R>(reader: io::BufReader<R>) -> Box<UciStream> where R: AsyncRead + Unpin + Sync + Send + 'static {
    let stream = reader.lines()
        .map_ok(|line| parse_with_unknown(&(line + "\n")))
        .map_ok(|msg_list| msg_list[0].clone())
        ;

    Box::new(stream)
}

pub fn stdin_msg_stream() -> Box<UciStream> {
    from_reader(io::BufReader::new(io::stdin()))

}

pub fn stdout_msg_sink() -> Box<UciSink> {
    let sink = io::stdout().into_sink().with(|msg: UciMessage| {
        ready(Ok(ByteVecUciMessage::from(msg))).boxed()
    });
    Box::new(sink)
}

pub async fn run_loops(
    mut inbound_source: Box<UciStream>,
    inbound_consumer: UciTrySender,
    mut outbound_source: UciReceiver,
    mut outbound_consumer: Box<UciSink>) {
    let inb = async {
        loop {
            let msg_result = inbound_source.try_next().await;
            if let Ok(msg_opt) = msg_result {
                if msg_opt.is_none() {
                    break;
                } else {
                    inbound_consumer.unbounded_send(Ok(msg_opt.unwrap()));
                }
            } else {
                inbound_consumer.unbounded_send(Err(msg_result.err().unwrap()));
            }

        }
    };

    let outb = async {
        while let Some(msg) = StreamExt::next(&mut outbound_source).await {
            println!("WRITING: {}", msg);
            outbound_consumer.send(msg).await;
        }
    };

    join!(inb, outb);
}

pub async fn run_std_loops(inbound_consumer: UciTrySender, mut outbound_source: UciReceiver) {
    run_loops(stdin_msg_stream(), inbound_consumer, outbound_source, stdout_msg_sink()).await;
}

pub fn new_channel() -> (UciSender, UciReceiver) {
    unbounded::<UciMessage>()
}

pub fn new_try_channel() -> (UciTrySender, UciTryReceiver) {
    unbounded::<io::Result<UciMessage>>()
}



#[cfg(test)]
mod tests {
    use futures::task::SpawnExt;

    use super::*;

    #[test]
    fn test_run_in_loop() {
        block_on(async {
            let mut msg_stream = stdin_msg_stream();
            while let Ok(msg_opt) = msg_stream.try_next().await {
                let msg = msg_opt.unwrap();
                println!("MSG RECEIVED VIA STREAM: {}", msg);
            }
        });
    }

    #[test]
    fn test_run_loops() {
        let (itx, mut irx) = new_try_channel();
        let (otx, orx) = new_channel();

        otx.unbounded_send(UciMessage::Uci);

        let rec = async {
            while let (Ok(incoming)) = TryStreamExt::try_next(&mut irx).await {
                println!("Handling received message: {}", incoming.unwrap())
            }
        };

        block_on(async {
            otx.unbounded_send(UciMessage::UciOk);
            join!(run_std_loops(itx, orx), rec);
        });
    }

}
