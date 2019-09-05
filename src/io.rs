use std::error::Error;

use async_std::io;
use async_std::task::block_on;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::join;
use vampirc_uci::{ByteVecUciMessage, parse_with_unknown, Serializable, UciMessage};

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
        });

    }

    Ok(())
}



#[cfg(test)]
mod tests {
    use futures::{StreamExt, TryStreamExt};

    use super::*;

    async fn process_rx(mut rx: UnboundedReceiver<UciMessage>) {
        loop {
            let m: UciMessage = rx.next().await.unwrap();
            println!("HERE WE ARE PROCESS: {}", m);
        }

    }

    #[test]
    fn test_run_in_loop() {
        let (tx, mut rx) = unbounded();
        block_on(async {
            let r = run_stdin_loop(tx);
            let p = process_rx(rx);
            join!(p, r);
        });
    }

}
