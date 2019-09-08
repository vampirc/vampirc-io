#![cfg(feature = "queue")]

use std::any::Any;
use std::time::Duration;

use async_std::io as aio;
use async_std::prelude::Future;
use async_std::task;
use crossbeam::queue::SegQueue;
use futures::{join, SinkExt, StreamExt, TryStreamExt};
use futures::channel::mpsc::{unbounded, UnboundedSender};
use vampirc_uci::UciMessage;

use crate::command::Command;
use crate::io::{new_channel, new_try_channel, run_std_loops, UciReceiver, UciSender, UciTryReceiver};

pub type CmdQueue = SegQueue<Box<dyn Command>>;
pub type UciQueue = SegQueue<UciMessage>;

pub fn new_cmd_queue() -> CmdQueue {
    SegQueue::new()
}

pub fn new_uci_queue() -> UciQueue {
    SegQueue::new()
}



pub async fn run_std_with_queues(inbound: &CmdQueue, outbound: &UciQueue, mut error_ch: UnboundedSender<aio::Error>) {
    let (itx, mut irx) = new_try_channel();
    let (mut otx, orx) = new_channel();

    let rec = async {
        loop {
            let incoming_res = TryStreamExt::try_next(&mut irx).await;
            if let Ok(incoming_opt) = incoming_res {
                if let Some(incoming) = incoming_opt {
                    inbound.push(Box::new(incoming));
                } else {
                    irx.close();
                    break;
                }
            } else {
                if let Err(send_err) = SinkExt::send(&mut error_ch, incoming_res.err().unwrap()).await {
                    eprintln!("Error while sending error report, closing: {}", send_err);
                    error_ch.close_channel();
                    break;
                }
            }
        }
    };

    let snd = async {
        loop {

            while let Ok(msg) = outbound.pop() {
                if let Err(send_err) = SinkExt::send(&mut otx, msg).await {
                    eprintln!("Error while sending message to output, closing: {}", send_err);
                    otx.close_channel();
                    return;
                }

                task::sleep(Duration::from_millis(25)).await;
            }

            task::sleep(Duration::from_millis(50)).await;
        }

    };


    join!(run_std_loops(itx, orx), rec, snd);
}

pub async fn run_std_with_inbound_queue(inbound: &CmdQueue, outbound_source: UciReceiver, mut error_ch: UnboundedSender<aio::Error>) {
    let (itx, mut irx) = new_try_channel();

    let rec = async {
        loop {
            println!("QRL");
            let incoming_res = TryStreamExt::try_next(&mut irx).await;
            println!("QRL AWAITED");
            if let Ok(incoming_opt) = incoming_res {
                if let Some(incoming) = incoming_opt {
                    inbound.push(Box::new(incoming));
                } else {
                    irx.close();
                    break;
                }
            } else {
                if let Err(send_err) = SinkExt::send(&mut error_ch, incoming_res.err().unwrap()).await {
                    eprintln!("Error while sending error report, closing: {}", send_err);
                    error_ch.close_channel();
                    break;
                }
            }
        }
    };


    join!(run_std_loops(itx, outbound_source), rec);
}

//fn build_inbound_future<'a>(inbound: &'a CmdQueue, irx: &'a mut UciTryReceiver,  mut error_ch: &'a mut UnboundedSender<aio::Error>) -> impl Future<Output=()>  + 'a{
//     async {
//        loop {
//            println!("QRL");
//            let incoming_res = TryStreamExt::try_next(irx).await;
//            println!("QRL AWAITED");
//            if let Ok(incoming_opt) = incoming_res {
//                if let Some(incoming) = incoming_opt {
//                    inbound.push(Box::new(incoming));
//                } else {
//                    irx.close();
//                    break;
//                }
//            } else {
//                if let Err(send_err) = SinkExt::send(&mut error_ch, incoming_res.err().unwrap()).await {
//                    eprintln!("Error while sending error report, closing: {}", send_err);
//                    error_ch.close_channel();
//                    break;
//                }
//            }
//        }
//    }
//
//}

#[cfg(test)]
mod tests {
    use async_std::task::block_on;

    use super::*;

    #[test]
    fn run_queue_loop() {
        let cq = new_cmd_queue();
        let uq = new_uci_queue();
        let (err_ch, _er) = unbounded::<aio::Error>();

        uq.push(UciMessage::Uci);
        uq.push(UciMessage::go_ponder());

        block_on(async {
            run_std_with_queues(&cq, &uq, err_ch).await;
        })
    }

    #[test]
    fn run_inb_queue_loop() {
        let cq = new_cmd_queue();
        let (err_ch, _er) = unbounded::<aio::Error>();
        let (mut otx, orx) = new_channel();


        block_on(async {
            otx.send(UciMessage::Uci).await;
            otx.send(UciMessage::go_ponder()).await;
            run_std_with_inbound_queue(&cq, orx, err_ch).await;
        })
    }
}