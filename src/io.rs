use std::error::Error;
use std::io;
use std::pin::Pin;

use futures::{AsyncBufReadExt, AsyncWriteExt, join, Stream, StreamExt, TryStreamExt};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::io::{AllowStdIo, BufReader, BufWriter, IntoSink};
use futures::task::Context;
use vampirc_uci::{ByteVecUciMessage, parse_with_unknown, Serializable, UciMessage};

pub type UciChannel = (UnboundedSender<UciMessage>, UnboundedReceiver<UciMessage>);
pub type UciSink = UnboundedSender<UciMessage>;
pub type UciStream = UnboundedReceiver<UciMessage>;

pub async fn run_inbound_loop(tx: UciSink) -> Result<(), Box<dyn Error>> {
    loop {
        let msg_str = read_stdin().await?;
        let msg_list = parse_with_unknown(msg_str.as_str());

        for i in 0..msg_list.len() {
            let msg = msg_list[i].clone();
            println!("MSG: {}", msg);
            tx.unbounded_send(msg)?;
        }
    }

    Ok(())
}

pub async fn run_outbound_loop(mut rx: UciStream) -> Result<(), Box<dyn Error>> {
    let mut stdout = BufWriter::new(AllowStdIo::new(io::stdout()));
    loop {
        let msg: UciMessage = rx.try_next()?.unwrap();
        let msg_str = msg.serialize();
        println!("WRITING {}", msg_str.as_str());
        let b = msg_str.as_bytes();
        stdout.write_all(b).await?;
    }

    Ok(())
}

async fn read_stdin() -> Result<String, Box<dyn Error>> {
    println!("READ STDIN");
    let din = io::stdin();
    let mut stdin = BufReader::new(AllowStdIo::new(din));
    let mut s: String = String::new();
    let len = stdin.read_line(&mut s).await?;
    Ok(s[..len].to_owned())
}



#[cfg(test)]
mod tests {
    use futures::{TryStream, TryStreamExt};
    use futures::executor;

    use super::*;

    async fn process_incoming(mut rrx: UciStream) -> Result<(), Box<dyn Error>> {
        println!("PROCESS INCOMING START");
        while let Some(msg) = rrx.next().await {
            println!("RECEIVED MSG: {}", msg);
        }
        println!("PROCESS INCOMING DONE");

        Ok(())
    }

    async fn print_num() {
        for i in 0..100 {
            println!("III: {}", i);
        }
    }

    #[test]
    fn test_run_in_loop() {
        executor::block_on(async {
            let (rtx, mut rrx) = unbounded::<UciMessage>();
            let pi = process_incoming(rrx);
            let il = run_inbound_loop(rtx);
            join!(pi, il);
        });
    }

    #[test]
    fn test_run_out_loop() {
        executor::block_on(async {
            let (stx, mut srx) = unbounded::<UciMessage>();
            stx.unbounded_send(UciMessage::Uci);
            stx.unbounded_send(UciMessage::UciOk);
            let il = run_outbound_loop(srx);
            let pn = print_num();
            join!(pn, il);
        });
    }
}