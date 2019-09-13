#![cfg(feature = "command")]

use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;
use std::pin::Pin;

use async_std::future::ready;
use async_std::io;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::stream::{Stream, StreamExt};
use vampirc_uci::{ByteVecUciMessage, UciMessage};

use crate::io::UciTryReceiver;

pub trait CmdObj: Display + Debug + Send + Sync + Unpin {

}

#[derive(Debug)]
pub enum Command {
    UciMessage(UciMessage),
    Error(io::Error),
    InternalCommand(Box<dyn CmdObj>),
    Uncatalogued(Box<dyn CmdObj>),
}

unsafe impl Send for Command {}

unsafe impl Sync for Command {}

impl Unpin for Command {}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Command::UciMessage(msg) => write!(f, "{}", msg),
            Command::Error(err) => write!(f, "{}", err),
            Command::InternalCommand(cmd) | Command::Uncatalogued(cmd) => write!(f, "{}", *cmd),
        }
    }
}

impl From<UciMessage> for Command {
    fn from(m: UciMessage) -> Self {
        Command::UciMessage(m)
    }
}

impl From<ByteVecUciMessage> for Command {
    fn from(m: ByteVecUciMessage) -> Self {
        Command::from(m.message)
    }
}

impl From<io::Error> for Command {
    fn from(err: io::Error) -> Self {
        Command::Error(err)
    }
}


impl Command {
    pub fn new_internal_cmd(c: Box<dyn CmdObj>) -> Command {
        Command::InternalCommand(c)
    }

    pub fn new_uncatalogued(c: Box<dyn CmdObj>) -> Command {
        Command::Uncatalogued(c)
    }
}


pub type CmdSender = UnboundedSender<Command>;
pub type CmdReceiver = UnboundedReceiver<Command>;

pub fn new_cmd_channel() -> (CmdSender, CmdReceiver) {
    unbounded()
}

pub fn as_cmd_stream(msg_rcv: UciTryReceiver) -> Pin<Box<impl Stream<Item=Command>>> {
    let mut rs = msg_rcv.filter_map(|msg: io::Result<UciMessage>| {
        if let Ok(m) = msg {
            ready(Some(Command::from(m)))
        } else {
            let err: io::Error = msg.err().unwrap();
            ready(Some(Command::from(err)))
        }
    });

    Box::pin(rs)
}