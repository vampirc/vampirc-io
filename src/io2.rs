use std::pin::Pin;

use async_std::io::{Stdin, stdin, Stdout, stdout};
use async_std::prelude::*;
use futures::{Future, Stream, StreamExt};
use futures::task::{Context, Poll};
use vampirc_uci::{parse_with_unknown, Serializable, UciMessage};

pub trait UciConsumer: Sync + Send {
    fn consume(&self, message: &UciMessage);
}

#[derive(Debug)]
pub struct GuiToEngineSync {
    std_in: Stdin,
    std_out: Stdout,
}

impl GuiToEngineSync {
    pub fn new() -> GuiToEngineSync {
        GuiToEngineSync {
            std_in: stdin(),
            std_out: stdout(),
        }
    }

    async fn next_line(&self) -> String {
        let mut s = String::with_capacity(1024);
        self.std_in.read_line(&mut s).await.unwrap();
        s
    }

    pub async fn next_message(&self) -> UciMessage {
        let line = self.next_line().await;
        let msg_list = parse_with_unknown(line.as_str());
        if msg_list.is_empty() || msg_list.len() > 1 {
            panic!("Expected to produce exactly one UciMessage");
        }

        msg_list[0].clone()
    }

    pub async fn send_message(&self, message: &UciMessage) {
        let mut handle = self.std_out.lock().await;
        handle.write_all(message.serialize().as_bytes()).await.unwrap();
    }

    pub async fn run_accept_loop(&mut self, consumer: &impl UciConsumer) {
        while let msg = self.next().await.unwrap() {
            consumer.consume(&msg)
        }
    }
}

impl Stream for GuiToEngineSync {
    type Item = UciMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let nm = self.next_message();
        let pin_mut_poll = Box::pin(nm).as_mut().poll(cx);

        match pin_mut_poll {
            Poll::Pending => Poll::Pending,
            Poll::Ready(message) => Poll::Ready(Some(message))
        }
    }
}

