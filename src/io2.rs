use std::pin::Pin;

use async_std::io::{BufReader, BufWriter, Stdin, stdin, Stdout, stdout};
use async_std::prelude::*;
use futures::{Future, join, Stream, StreamExt};
use futures::future::BoxFuture;
use futures::task::{Context, Poll};
use vampirc_uci::{parse_with_unknown, Serializable, UciMessage};

#[derive(Debug)]
pub struct StdinStream {
    std_in: BufReader<Stdin>
}

impl StdinStream {
    pub fn new() -> StdinStream {
        StdinStream {
            std_in: BufReader::new(stdin())
        }
    }

    async fn next_line(&mut self) -> async_std::io::Result<String> {
        let mut s = String::new();
        self.std_in.read_line(&mut s).await?;
        Ok(s)
    }

    pub async fn next_message(&mut self) -> UciMessage {
        let mut line = self.next_line().await;
        while line.is_err() {
            line = self.next_line().await;
        }

        let msg_list = parse_with_unknown(line.unwrap().as_str());
        if msg_list.is_empty() || msg_list.len() > 1 {
            panic!("Expected to produce exactly one UciMessage");
        }

        msg_list[0].clone()
    }
}

impl Stream for StdinStream {
    type Item = UciMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let nm = self.next_message();
        let pin_mut_poll = Box::pin(nm).as_mut().poll(cx);

        match pin_mut_poll {
            Poll::Pending => Poll::Pending,
            Poll::Ready(message) => Poll::Ready(Some(message))
        }
    }
}

#[derive(Debug)]
pub struct StdoutSink {
    std_out: BufWriter<Stdout>
}

impl StdoutSink {
    pub fn new() -> StdoutSink {
        StdoutSink {
            std_out: BufWriter::new(stdout())
        }
    }

    pub async fn send_message(&mut self, message: &UciMessage) {
        let s = message.serialize() + "\r\n";
        self.std_out.write_all(s.as_bytes()).await.unwrap();
    }
}

