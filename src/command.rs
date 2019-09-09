use std::fmt::{Debug, Display};
use std::hash::Hash;

use async_std::future::ready;
use async_std::io;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::stream::{Stream, StreamExt};
use vampirc_uci::{ByteVecUciMessage, UciMessage};

use crate::io::UciTryReceiver;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum CommandType {
    UciMessage,
    InternalCommand,
    Uncatalogued,
}


impl Default for CommandType {
    fn default() -> Self {
        CommandType::Uncatalogued
    }
}

pub trait Command: Display + Debug + Send + Sync {
    fn get_type(&self) -> CommandType;
}

impl Command for UciMessage {
    #[inline]
    fn get_type(&self) -> CommandType {
        CommandType::UciMessage
    }
}

impl Command for ByteVecUciMessage {
    #[inline]
    fn get_type(&self) -> CommandType {
        CommandType::UciMessage
    }
}

pub type CmdSender = UnboundedSender<Box<dyn Command>>;
pub type CmdReceiver = UnboundedReceiver<Box<dyn Command>>;

pub fn new_cmd_channel() -> (CmdSender, CmdReceiver) {
    unbounded()
}

pub fn as_cmd_stream(msg_rcv: UciTryReceiver) -> impl Stream<Item=Box<dyn Command>> {
    let mut rs = msg_rcv.filter_map(|msg: io::Result<UciMessage>| {
        if let Ok(m) = msg {
            let b: Box<dyn Command> = Box::new(m);
            ready(Some(b))
        } else {
            ready(None)
        }
    });

    rs
}