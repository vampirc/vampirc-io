use std::error::Error;
use std::io;
use std::io::{BufRead, Read};
use std::pin::Pin;

use futures::{AsyncBufReadExt, AsyncWriteExt};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::io::{AllowStdIo, IntoSink};
use futures::task::Context;
use vampirc_uci::{ByteVecUciMessage, parse_with_unknown, UciMessage};

type UciChannel = (UnboundedSender<UciMessage>, UnboundedReceiver<UciMessage>);
type UciSink = UnboundedSender<UciMessage>;
type UciStream = UnboundedReceiver<UciMessage>;

pub async fn run_inbound_loop(tx: UciSink) -> Result<(), Box<dyn Error>> {
    loop {
        let msg_str = read_stdin().await?;
        let msg_list = parse_with_unknown(msg_str.as_str());

        for i in 0..msg_list.len() {
            let msg = msg_list[i].clone();
            tx.unbounded_send(msg)?;
        }
    }

    Ok(())
}

async fn read_stdin() -> Result<String, Box<dyn Error>> {
    let din = io::stdin();
    let lock = din.lock();
    let mut stdin = AllowStdIo::new(lock);
    let mut s: String = String::new();
    let len = AsyncBufReadExt::read_line(&mut stdin, &mut s).await?;
    Ok(s[..len].to_owned())
}



#[cfg(test)]
mod tests {
    use futures::executor;

    use super::*;
}