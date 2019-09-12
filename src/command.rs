use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;
use std::pin::Pin;

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
    Error,
    Uncatalogued
}


impl Default for CommandType {
    fn default() -> Self {
        CommandType::Uncatalogued
    }
}

pub trait Command: Display + Debug + Send + Sync + Unpin {
    fn get_type(&self) -> CommandType;
}

#[derive(Debug)]
pub struct CmdError(pub io::Error);

impl Command for CmdError {
    #[inline]
    fn get_type(&self) -> CommandType {
        CommandType::Error
    }
}

impl Display for CmdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl From<io::Error> for CmdError {
    fn from(err: io::Error) -> Self {
        CmdError(err)
    }
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

pub fn as_cmd_stream(msg_rcv: UciTryReceiver) -> Pin<Box<impl Stream<Item=Box<dyn Command>>>> {
    let mut rs = msg_rcv.filter_map(|msg: io::Result<UciMessage>| {
        if let Ok(m) = msg {
            let b: Box<dyn Command> = Box::new(m);
            ready(Some(b))
        } else {
            let err: io::Error = msg.err().unwrap();
            let b: Box<dyn Command> = Box::new(CmdError::from(err));
            ready(Some(b))
        }
    });

    Box::pin(rs)
}