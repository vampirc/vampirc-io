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
pub type UnboundedSink = UnboundedSender<UciMessage>;
pub type UciChannel = (UnboundedSender<UciMessage>, UnboundedReceiver<UciMessage>);



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

pub async fn run_loops(inbound_consumer: UnboundedSender<UciMessage>, mut outbound_source: UnboundedReceiver<UciMessage>) {
    let mut stdin = stdin_msg_stream();
    let inb = async {
        while let Ok(msg_opt) = stdin.try_next().await {
            let msg = msg_opt.unwrap();
            println!("READING: {}", msg);
            inbound_consumer.unbounded_send(msg);
        }
    };

    let mut stdout = stdout_msg_sink();
    let outb = async {
        while let Some(msg) = StreamExt::next(&mut outbound_source).await {
            println!("WRITING: {}", msg);
            stdout.send(msg).await;
        }
    };

    join!(inb, outb);
}



#[cfg(test)]
mod tests {
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
        let (itx, irx) = unbounded::<UciMessage>();
        let (otx, orx) = unbounded::<UciMessage>();

        otx.unbounded_send(UciMessage::Uci);

        block_on(async {
            otx.unbounded_send(UciMessage::UciOk);
            run_loops(itx, orx).await;
        });
    }

}
