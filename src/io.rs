use std::ops::DerefMut;
use std::sync::Arc;

use async_std::io;
use async_std::io::{BufRead, ErrorKind};
use async_std::sync::{Mutex, RwLock};
use async_std::task::block_on;
use futures::{AsyncRead, future, FutureExt, join, SinkExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use vampirc_uci::{ByteVecUciMessage, MessageList, parse_strict, parse_with_unknown, Serializable, UciMessage};

pub type UciStream = dyn Stream<Item=Result<UciMessage, io::Error>> + Unpin + Send + Sync;

pub async fn run_stdin() -> io::Result<UnboundedReceiver<UciMessage>> {
    let (tx, rx) = unbounded::<UciMessage>();

    run_stdin_loop(tx);

    Ok(rx)
}

pub async fn run_stdin_loop(tx: UnboundedSender<UciMessage>) -> io::Result<()> {
    let stdin = io::stdin();

    loop {
        let mut raw_str = String::new();

        let len = stdin.read_line(&mut raw_str).await?;
        let msg_str: &str = &raw_str[0..len];
        let msg_list = parse_with_unknown(msg_str).into_iter();
        msg_list.for_each(|msg| {
            println!("READ: {}", msg);
            tx.unbounded_send(msg);
        })

    }

    Ok(())
}

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

}
