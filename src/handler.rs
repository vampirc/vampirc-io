use std::fmt::Display;
use std::io;
use std::iter::Iterator;

use crossbeam::queue::ArrayQueue;
use futures::{lazy, Future, Sink, Stream};
use tokio::io::{AsyncRead, AsyncWrite, ErrorKind};
use vampirc_uci::{CommunicationDirection, UciMessage};

use crate::UciStream;

const Q_SIZE: usize = 1000;

pub struct StreamHolder<S, F, E>
where
    S: AsyncRead + AsyncWrite + Send + Sync + 'static,
    F: Fn(UciMessage) + Send + Copy + 'static,
    E: Fn(&io::Error, CommunicationDirection) + Send + Copy + 'static,
{
    pub frame: UciStream<S>,
    pub receiver: F,
    pub outbound: Box<Stream<Item = UciMessage, Error = io::Error> + Send>,
    pub error: E,
}

impl<S, F, E> StreamHolder<S, F, E>
where
    S: AsyncRead + AsyncWrite + Send + Sync + 'static,
    F: Fn(UciMessage) + Send + Copy + 'static,
    E: Fn(&io::Error, CommunicationDirection) + Send + Copy + 'static,
{
}

pub trait MsgHandler<S, F, E>
where
    S: AsyncRead + AsyncWrite + Send + Sync + 'static,
    F: Fn(UciMessage) + Send + Copy + 'static,
    E: Fn(&io::Error, CommunicationDirection) + Send + Copy + 'static,
{
    fn init_and_run(self, sh: StreamHolder<S, F, E>);

    fn send(&self, m: UciMessage) -> Result<(), io::Error>;
}

struct QueueBasedHandler<S, F, E>
where
    S: AsyncRead + AsyncWrite + Send + Sync + 'static,
    F: Fn(UciMessage) + Send + Copy + 'static,
    E: Fn(&io::Error, CommunicationDirection) + Send + Copy + 'static,
{
    holder: StreamHolder<S, F, E>,
    inbound: ArrayQueue<UciMessage>,
    outbound: ArrayQueue<UciMessage>,
}

impl<S, F, E> QueueBasedHandler<S, F, E>
where
    S: AsyncRead + AsyncWrite + Send + Sync + 'static,
    F: Fn(UciMessage) + Send + Copy + 'static,
    E: Fn(&io::Error, CommunicationDirection) + Send + Copy + 'static,
{
    pub fn with_holder(holder: StreamHolder<S, F, E>) -> QueueBasedHandler<S, F, E> {
        QueueBasedHandler {
            holder,
            inbound: ArrayQueue::new(Q_SIZE),
            outbound: ArrayQueue::new(Q_SIZE),
        }
    }
}

impl<S, F, E> MsgHandler<S, F, E> for QueueBasedHandler<S, F, E>
where
    S: AsyncRead + AsyncWrite + Send + Sync + 'static,
    F: Fn(UciMessage) + Send + Copy + 'static,
    E: Fn(&io::Error, CommunicationDirection) + Send + Copy + 'static,
{
    fn init_and_run(self, sh: StreamHolder<S, F, E>) {
        //        let (sink, stream) = sh.frame.split();
        //        let error_func = sh.error;
        //        let handle_func = sh.receiver;
        //        let in_q = self.inbound;
        //
        //        let proc_in = stream
        //            .for_each(move |m: UciMessage| {
        //                in_q.push(m);
        //                Ok(())
        //            })
        //            .map_err(move |e| {
        //                (error_func)(&e, CommunicationDirection::GuiToEngine);
        //            });
        //
        //        let proc_handle = lazy(move || {
        //            loop {
        //                //                if let Ok(m) = in_q.pop() {
        //                //                    (handle_func)(m);
        //                //                }
        //            }
        //            Ok(())
        //        })
        //        .map_err(move |e| {
        //            (error_func)(&e, CommunicationDirection::GuiToEngine);
        //        });
        //        ;
        //
        //        tokio::run(lazy(move || {
        //            tokio::spawn(proc_in);
        //            tokio::spawn(proc_handle);
        //            Ok(())
        //        }));
    }

    fn send(&self, m: UciMessage) -> Result<(), io::Error> {
        let r = self.outbound.push(m);

        if let Ok(msg) = r {
            return Result::Ok(());
        }

        Result::Err(io::Error::new(ErrorKind::WouldBlock, r.err().unwrap()))
    }
}
