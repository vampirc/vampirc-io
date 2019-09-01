use std::io;
use std::pin::Pin;

use crossbeam::queue::SegQueue;
use futures::{AsyncBufReadExt, AsyncWriteExt, FutureExt, join, Poll, Sink, SinkExt, Stream, StreamExt};
use futures::io::{AllowStdIo, IntoSink};
use futures::task::Context;
use vampirc_uci::{ByteVecUciMessage, parse_with_unknown, UciMessage};

async fn async_write() {
    let mut stdout = AllowStdIo::new(io::stdout());
    stdout.write_all("Whatever\n".as_bytes()).await;
}


pub async fn run_dispatcher(mut source: Pin<&mut dyn Stream<Item=UciMessage>>, mut destination: Pin<&mut dyn Sink<ByteVecUciMessage, Error=io::Error>>) {
    println!("RUN DISPATCHER");
    while let Some(msg) = source.next().await {
        println!("AWAITED SRC MSG: {}", msg);
        let bam = ByteVecUciMessage::from(msg);
        destination.send(bam).await;
    }
}

pub async fn dispatch_continuously(
    inbound_source: Pin<&mut dyn Stream<Item=UciMessage>>,
    inbound_destination: Pin<&mut dyn Sink<ByteVecUciMessage, Error=io::Error>>,
    outbound_source: Pin<&mut dyn Stream<Item=UciMessage>>,
    outbound_destination: Pin<&mut dyn Sink<ByteVecUciMessage, Error=io::Error>>,
) {
    let inbound_dispatch = run_dispatcher(inbound_source, inbound_destination);
    let outbound_dispatch = run_dispatcher(outbound_source, outbound_destination);

    join!(outbound_dispatch, inbound_dispatch);
}

pub async fn dispatch_default(inbound_destination: &mut UciMessageQueue, outbound_source: &mut UciMessageQueue) {
    let mut inbound_source = DispatcherStdinSource::default();
    let mut outbound_destination = DispatcherStdoutTarget::default().into_sink();

    let pin_inb_src = Pin::new(&mut inbound_source);
    let pin_out_dest = Pin::new(&mut outbound_destination);
    let pin_inb_dest = Pin::new(inbound_destination);
    let pin_out_src = Pin::new(outbound_source);

    dispatch_continuously(pin_inb_src, pin_inb_dest, pin_out_src, pin_out_dest).await;
}

#[derive(Debug)]
pub struct UciMessageQueue(pub SegQueue<UciMessage>);

impl Stream for UciMessageQueue {
    type Item = UciMessage;

    /// Attempt to resolve the next item in the stream.
    /// Retuns `Poll::Pending` if not ready, `Poll::Ready(Some(x))` if a value
    /// is ready, and never returns `Poll::Ready(None)` as the stream is never completed
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<UciMessage>> {
        println!("POLL: {}", self.0.len());
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

unsafe impl Sync for UciMessageQueue {}

unsafe impl Send for UciMessageQueue {}


impl UciMessageQueue {
    pub fn new() -> UciMessageQueue {
        UciMessageQueue::default()
    }
}

impl Sink<ByteVecUciMessage> for UciMessageQueue {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {

        // Our unbounded queue can't really fail to push, so we're always ready to do this
        Poll::Ready(Result::Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: ByteVecUciMessage) -> Result<(), Self::Error> {
        println!("Inserting message into queue: {}", item);
        self.0.push(item.into());
        Result::Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        // We don't really buffer
        Poll::Ready(Result::Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        // Nothing to do for close
        Poll::Ready(Result::Ok(()))
    }
}

#[derive(Debug)]
pub struct DispatcherStdoutTarget(AllowStdIo<io::Stdout>);

impl Unpin for DispatcherStdoutTarget {}

impl Default for DispatcherStdoutTarget {
    fn default() -> Self {
        DispatcherStdoutTarget(AllowStdIo::new(io::stdout()))
    }
}

unsafe impl Sync for DispatcherStdoutTarget {}

unsafe impl Send for DispatcherStdoutTarget {}

impl DispatcherStdoutTarget {
    pub fn into_sink(self) -> IntoSink<AllowStdIo<io::Stdout>, ByteVecUciMessage> {
        self.0.into_sink()
    }
}

#[derive(Debug)]
pub struct DispatcherStdinSource(AllowStdIo<io::BufReader<io::Stdin>>);

impl Unpin for DispatcherStdinSource {}

impl Default for DispatcherStdinSource {
    fn default() -> Self {
        DispatcherStdinSource(AllowStdIo::new(io::BufReader::new(io::stdin())))
    }
}

unsafe impl Sync for DispatcherStdinSource {}

unsafe impl Send for DispatcherStdinSource {}

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
    use std::sync::Arc;
    use std::thread::spawn;

    use futures::executor;

    use super::*;

    #[test]
    pub fn test_async_write() {
        executor::block_on(async {
            async_write().await;
        });
    }

    #[test]
    #[ignore]
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

    #[test]
//    #[ignore]
    pub fn test_run_dispatcher_inbound() {
        executor::block_on(async {
            let mut dqt = UciMessageQueue::default();
            let mut dss = DispatcherStdinSource::default();
            let src = unsafe { Pin::new_unchecked(&mut dss) };
            let tgt = unsafe { Pin::new_unchecked(&mut dqt) };
            run_dispatcher(src, tgt).await;
        });
    }

    #[test]
//    #[ignore]
    pub fn test_dispatch_default() {
        executor::block_on(async {
            let mut inq = UciMessageQueue::default();
            let mut ouq = UciMessageQueue::default();
            ouq.0.push(UciMessage::PonderHit);
            ouq.0.push(UciMessage::UciNewGame);
            //let ooq = Arc::clone(&ouq);

//            spawn(move || {
//                ooq.0.push(UciMessage::IsReady);
//            });

            dispatch_default(&mut inq, &mut ouq).await;


        });
    }
}