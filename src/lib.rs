#[cfg(test)]
extern crate bytes;
extern crate tokio;
#[cfg(feature = "vampirc-uci")]
extern crate vampirc_uci;

mod io;
mod parse;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}