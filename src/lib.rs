extern crate async_std;
extern crate futures;
extern crate vampirc_uci;

pub use async_std::io::Result;
pub use futures::join;

pub use crate::io::from_reader;
pub use crate::io::new_channel;
pub use crate::io::new_try_channel;
pub use crate::io::run_future;
pub use crate::io::run_loops;
pub use crate::io::run_std_loops;
pub use crate::io::stdin_msg_stream;
pub use crate::io::stdout_msg_sink;
pub use crate::io::UciReceiver;
pub use crate::io::UciSender;
pub use crate::io::UciSink;
pub use crate::io::UciStream;
pub use crate::io::UciTryReceiver;
pub use crate::io::UciTrySender;

mod io;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
