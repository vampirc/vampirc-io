#![cfg(feature = "command")]

use std::fmt::{Debug, Display, Error, Formatter};
use std::fs::read;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;

use async_std::future::ready;
use async_std::io;
use downcast_rs::Downcast;
use futures::{Future, FutureExt, join, SinkExt, TryFutureExt};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::stream::{Stream, StreamExt, TryStreamExt};
use vampirc_uci::{ByteVecUciMessage, UciMessage};

use crate::io::UciTryReceiver;
use crate::UciSender;

pub trait CmdObj: Display + Debug + Send + Sync + Unpin + Downcast + 'static {}
impl_downcast!(CmdObj);

#[derive(Debug)]
pub enum Command {
    UciMessage(UciMessage),
    Error(io::Error),
    InternalCommand(Arc<dyn CmdObj>),
    Uncatalogued(Arc<dyn CmdObj>),
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
    pub fn new_internal_cmd(c: Arc<dyn CmdObj>) -> Command {
        Command::InternalCommand(c)
    }

    pub fn new_uncatalogued(c: Arc<dyn CmdObj>) -> Command {
        Command::Uncatalogued(c)
    }
}


pub type CmdSender = UnboundedSender<Command>;
pub type CmdReceiver = UnboundedReceiver<Command>;

pub fn new_cmd_channel() -> (CmdSender, CmdReceiver) {
    unbounded()
}

pub async fn pipe_to_cmd_stream(msg_rcv: UciTryReceiver, mut cmd_snd: Pin<&mut CmdSender>) {
    let mut rs = msg_rcv.filter_map(|msg: io::Result<UciMessage>| {
        if let Ok(m) = msg {
            ready(Some(Command::from(m)))
        } else {
            let err: io::Error = msg.err().unwrap();
            ready(Some(Command::from(err)))
        }
    });

    let mut pin_rs = Box::pin(rs);

    let aas = async {
        while let Some(msg_cmd) = pin_rs.next().await {
            cmd_snd.send(msg_cmd).await;
        }
    };

    join!(aas);

}