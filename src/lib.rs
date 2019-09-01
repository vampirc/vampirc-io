#![feature(async_await)]
#[macro_use]
extern crate futures;
extern crate vampirc_uci;

mod io;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
