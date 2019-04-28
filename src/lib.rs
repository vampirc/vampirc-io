#[cfg(test)]
extern crate bytes;
extern crate tokio;
extern crate tokio_codec;
extern crate vampirc_uci;

pub use crate::io::run;
pub use crate::io::run_engine;

mod codec;
mod io;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}