#![feature(async_await)]

use std::error::Error;
use std::fs::read;
use std::io;
use std::io::BufRead;
use std::pin::Pin;

use crossbeam::queue::SegQueue;
use futures::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, executor, FutureExt, Poll, Sink, SinkExt, Stream, StreamExt};
use futures::io::{AllowStdIo, AsyncReadExt, IntoSink, Lines};
use futures::task::Context;
use vampirc_uci::{ByteVecUciMessage, parse_with_unknown, UciMessage};

async fn async_write() {
    let mut stdout = AllowStdIo::new(io::stdout());
    stdout.write_all("Whatever\n".as_bytes()).await;
}


pub async fn run_dispatcher(mut source: Pin<&mut dyn Stream<Item=UciMessage>>, mut destination: Pin<&mut dyn Sink<ByteVecUciMessage, Error=io::Error>>) {
    while let Some(msg) = source.next().await {
        let bam = ByteVecUciMessage::from(msg);
        destination.send(bam).await;
    }
}

#[derive(Debug)]
pub struct UciMessageQueue(pub SegQueue<UciMessage>);

impl Stream for UciMessageQueue {
    type Item = UciMessage;

    /// Attempt to resolve the next item in the stream.
    /// Retuns `Poll::Pending` if not ready, `Poll::Ready(Some(x))` if a value
    /// is ready, and never returns `Poll::Ready(None)` as the stream is never completed
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<UciMessage>> {
        if self.0.is_empty() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        if let Ok(msg) = self.0.pop() {
            return Poll::Ready(Some(msg));
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

impl Default for UciMessageQueue {
    fn default() -> Self {
        UciMessageQueue(SegQueue::default())
    }
}

impl UciMessageQueue {
    pub fn new() -> UciMessageQueue {
        UciMessageQueue::default()
    }
}

pub struct DispatcherStdoutTarget(AllowStdIo<io::Stdout>);

impl Unpin for DispatcherStdoutTarget {}

impl Default for DispatcherStdoutTarget {
    fn default() -> Self {
        DispatcherStdoutTarget(AllowStdIo::new(io::stdout()))
    }
}

impl DispatcherStdoutTarget {
    pub fn into_sink(self) -> IntoSink<AllowStdIo<io::Stdout>, ByteVecUciMessage> {
        self.0.into_sink()
    }
}

pub struct DispatcherStdinSource(AllowStdIo<io::BufReader<io::Stdin>>);

impl Unpin for DispatcherStdinSource {}

impl Default for DispatcherStdinSource {
    fn default() -> Self {
        DispatcherStdinSource(AllowStdIo::new(io::BufReader::new(io::stdin())))
    }
}

impl DispatcherStdinSource {
    pub fn into_stream(self) {
        let s = AsyncBufReadExt::lines(self.0)
            .map(|l| { l.unwrap() })
            .map(|l| { parse_with_unknown(l.as_str()) })
            ;
    }
}

impl Stream for DispatcherStdinSource {
    type Item = UciMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut line = String::new();
        let mut rl = AsyncBufReadExt::read_line(&mut self.0, &mut line);
        let poll_result = rl.poll_unpin(cx);

        if !poll_result.is_ready() {
            return Poll::Pending;
        }

        let msgs = parse_with_unknown(line.as_str());

        if msgs.len() < 1 {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        // TODO should probably buffer the others if there is more than one
        let msg = msgs[0].clone();

        Poll::Ready(Option::Some(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_async_write() {
        executor::block_on(async {
            async_write().await;
        });
    }

    #[test]
    pub fn test_run_dispatcher() {
        executor::block_on(async {
            let mut dqs = UciMessageQueue::default();
            dqs.0.push(UciMessage::Uci);
            dqs.0.push(UciMessage::UciOk);
            let mut dst = DispatcherStdoutTarget::default().into_sink();
            let src = unsafe { Pin::new_unchecked(&mut dqs) };
            let tgt = unsafe { Pin::new_unchecked(&mut dst) };
            run_dispatcher(src, tgt).await;
        });
    }
}