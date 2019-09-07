#![cfg(feature = "queue")]

use std::any::Any;

use async_std::io as aio;
use crossbeam::queue::SegQueue;
use futures::{join, SinkExt, TryStreamExt};
use futures::channel::mpsc::UnboundedSender;
use vampirc_uci::UciMessage;

use crate::command::Command;
use crate::io::{new_channel, new_try_channel, run_std_loops};

pub type CmdQueue = SegQueue<Box<dyn Command>>;
pub type UciQueue = SegQueue<UciMessage>;

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
            }
        }
    };


    join!(run_std_loops(itx, orx), rec, snd);
}