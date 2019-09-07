extern crate async_std;
#[cfg(feature = "queue")]
extern crate crossbeam;
extern crate futures;
extern crate vampirc_uci;

mod io;
mod command;
#[cfg(feature = "queue")]
mod queue;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
