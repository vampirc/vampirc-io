#[cfg(test)]
extern crate bytes;
extern crate crossbeam;
extern crate futures;
extern crate tokio;
extern crate tokio_codec;
extern crate vampirc_uci;

pub use crate::io::new_uci_engine_stream;
pub use crate::io::run;
pub use crate::io::run_engine;
pub use crate::io::UciEngineStream;
pub use crate::io::UciStream;

mod codec;
mod io;
mod handler;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
