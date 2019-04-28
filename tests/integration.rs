use std::io;

use tokio::codec::{FramedRead, LinesCodec};
use tokio::io::stdin;
use tokio::prelude::{Future, Stream};
use vampirc_uci::UciMessage;

use vampirc_io::run_engine;

pub fn setup() {
    println!("Setting up fixtures");
}

pub fn teardown() {
    println!("Tearing down");
}

#[test]
fn sum_test() {
    assert_eq!(6 + 8, 14);
}

#[test]
#[ignore]
fn test_interactive_stdin_read_async() {
    println!("Enter some input >>> ");

    let frs = FramedRead::new(stdin(), LinesCodec::new());
    let proc = frs.for_each(|m| {
        println!("Message: {}", m);
        Ok(())
    })
        .map_err(|e| {
            println!("ERROR: {}", e);
        })

        ;

    tokio::run(proc);
}

#[test]
#[ignore]
fn test_pass_func() {
    run_engine(callback1, err_callback);
}

fn callback1(m: &UciMessage) {
    println!("M: {}", *m);
}

fn err_callback(e: &io::Error) {
    println!("Error handling: {}", e);
}