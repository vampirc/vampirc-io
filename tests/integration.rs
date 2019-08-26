use std::io;

use vampirc_uci::UciMessage;

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
//    println!("Enter some input >>> ");
//
//    let frs = FramedRead::new(stdin(), LinesCodec::new());
//    let proc = frs.for_each(|m| {
//        println!("Message: {}", m);
//        Ok(())
//    })
//        .map_err(|e| {
//            println!("ERROR: {}", e);
//        })
//
//        ;
//
//    tokio::run(proc);
}

#[test]
#[ignore]
fn test_pass_func() {
    // run_engine(callback1, err_callback);
}