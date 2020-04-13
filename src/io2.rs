use async_std::io::{Stdin, stdin, Stdout, stdout};
use async_std::prelude::*;
use vampirc_uci::{parse_with_unknown, Serializable, UciMessage};

#[derive(Debug)]
pub struct GuiToEngineSync {
    std_in: Stdin,
    std_out: Stdout,
}

impl GuiToEngineSync {
    pub fn new() -> GuiToEngineSync {
        GuiToEngineSync {
            std_in: stdin(),
            std_out: stdout(),
        }
    }

    async fn next_line(&self) -> String {
        let mut s = String::with_capacity(1024);
        self.std_in.read_line(&mut s).await.unwrap();
        s
    }

    pub async fn next_message(&self) -> UciMessage {
        let line = self.next_line().await;
        let msg_list = parse_with_unknown(line.as_str());
        if msg_list.is_empty() || msg_list.len() > 1 {
            panic!("Expected to produce exactly one UciMessage");
        }

        msg_list[0].clone()
    }

    pub async fn send_message(&self, message: &UciMessage) {
        let mut handle = self.std_out.lock().await;
        handle.write_all(message.serialize().as_bytes()).await.unwrap();
    }
}

