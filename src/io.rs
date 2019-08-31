#![feature(async_await)]

use std::error::Error;
use std::io;
use std::io::Stdout;
use std::ops::Deref;
use std::pin::Pin;

use crossbeam::queue::SegQueue;
use futures::{AsyncWriteExt, executor, Poll, Sink, SinkExt, Stream, StreamExt};
use futures::io::{AllowStdIo, AsyncReadExt, IntoSink};
use futures::task::Context;
use vampirc_uci::{ByteVecUciMessage, UciMessage};

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
pub struct DispatcherQueueSource(pub SegQueue<UciMessage>);

impl Stream for DispatcherQueueSource {
    type Item = UciMessage;

    /// Attempt to resolve the next item in the stream.
    /// Retuns `Poll::Pending` if not ready, `Poll::Ready(Some(x))` if a value
    /// is ready, and never returns `Poll::Ready(None)` as the stream is never completed
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<UciMessage>> {

        // TODO should we wake something?

        if let Ok(msg) = self.0.pop() {
            return Poll::Ready(Some(msg));
        }

        Poll::Pending
    }
}

impl Default for DispatcherQueueSource {
    fn default() -> Self {
        DispatcherQueueSource(SegQueue::default())
    }
}

impl DispatcherQueueSource {
    pub fn new() -> DispatcherQueueSource {
        DispatcherQueueSource::default()
    }
}

pub struct DispatcherStdioTarget(AllowStdIo<io::Stdout>);

impl Unpin for DispatcherStdioTarget {}

impl Default for DispatcherStdioTarget {
    fn default() -> Self {
        DispatcherStdioTarget(AllowStdIo::new(io::stdout()))
    }
}

impl DispatcherStdioTarget {
    pub fn into_sink(self) -> IntoSink<AllowStdIo<Stdout>, ByteVecUciMessage> {
        self.0.into_sink()
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
            let mut dqs = DispatcherQueueSource::default();
            dqs.0.push(UciMessage::Uci);
            dqs.0.push(UciMessage::UciOk);
            let mut dst = DispatcherStdioTarget::default().into_sink();
            let src = unsafe { Pin::new_unchecked(&mut dqs) };
            let mut tgt = unsafe { Pin::new_unchecked(&mut dst) };
            run_dispatcher(src, tgt).await;
        });
    }
}