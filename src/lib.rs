#![feature(async_await)]
#[macro_use]
#[feature(async_await, await_macro, futures_api)]
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
