use std::fmt::{Debug, Display};
use std::hash::Hash;

use vampirc_uci::{ByteVecUciMessage, UciMessage};

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

pub trait Command: Display + Debug {
    fn get_type(&self) -> CommandType;
}

impl Command for UciMessage {
    fn get_type(&self) -> CommandType {
        CommandType::UciMessage
    }
}

impl Command for ByteVecUciMessage {
    fn get_type(&self) -> CommandType {
        CommandType::UciMessage
    }
}