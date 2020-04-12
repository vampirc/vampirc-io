# vampirc-io [![Build Status](https://travis-ci.org/vampirc/vampirc-io.svg?branch=master)](https://travis-ci.org/vampirc/vampirc-io) [![Documentation Status](https://docs.rs/vampirc-io/badge.svg)](https://docs.rs/vampirc-io)

vampirc-io is a companion crate to [vampirc-uci](https://github.com/vampirc/vampirc-uci). While vampirc-uci handles parsing and serialization of UCI messages,
vampirc-io handles communication of said messages between the chess client and the chess engine – usually over standard input and standard output.

It does so using asynchronous facilities of the Rust language, most notably the [async-std](https://github.com/async-rs/async-std). The loop that reads
data from stdin and parses them into a stream of [UciMessages](https://docs.rs/vampirc-uci/0.8/vampirc_uci/uci/enum.UciMessage.html), and the loop that writes messages to stdout run in an asynchronous, non-blocking way.

**Info**: Since 0.3.0, this crate no longer requires the nightly Rust build, but it does require 1.39+, for async support. 

The UCI protocol is a way for a chess engine to communicate with a chessboard GUI, such as [Scid vs. PC](http://scidvspc.sourceforge.net/).

The [Vampirc Project](https://vampirc.kejzar.si) is a chess engine and chess library suite, written in Rust. It is named for the
Slovenian grandmaster [Vasja Pirc](https://en.wikipedia.org/wiki/Vasja_Pirc), and, I guess, vampires? I dunno.

To use the crate, declare a dependency on it in your Cargo.toml file:


## Usage

Declare dependency in your `Cargo.toml`:
```toml
[dependencies]
vampirc-io = "0.3"
```

Then reference the `vampirc_io` crate in your crate root:
```rust
extern crate vampirc_io;
```

Imports:
```rust
use vampirc_io as vio;
use vampirc_uci::UciMessage;
```

Create an inbound futures channel - a channel for incoming UCI messages (those coming in from stdin):
```rust
let (itx, irx) = vio::new_try_channel();
```

Create an outbound futures channel - a channel for outgoing UCI messages (those output to stdout):
```rust
let (otx, orx) = vio::new_channel();
```

Write an async function that handles the incoming messages, something like:
```rust
async fn process_message(engine: Arc<Engine>, mut msg_stream: Pin<Box<impl Stream<Item = io::Result<UciMessage>>>>, msg_handler: &dyn MsgHandler, msg_sender: &vio::UciSender) {

    while let Some(msg_r) =  msg_stream.next().await {
        if let Ok(msg) = msg_r {
            log_message(&msg);
            msg_handler.handle_msg(engine.as_ref(), &msg, msg_sender);
        } else {
            log_error(msg_r.err().unwrap());
        }
    }

}
```
The  `msg_stream` parameter is your stream of incoming messages - the receiving end of the inbound channel, or the `irx` variable from the inbound channel
declaration. `msg_sender` is the sending end of the outbound channel where you will send your UCI messages, or `otx` from the outbound channel declaration.

And, lastly, and most importantly, run  stdin/stdout reading/writing loops (the `vio::run_std_loops` asynchronous function) and your process_message handler asynchronously
in your `main` function, using the `join!` macro:
```rust
vio::run_future(async {

        vio::join!(vio::run_std_loops(itx, orx), process_cmd(engine2, msg_stream, &msg_handler, &otx));
});
```

## Backing infrastructure and future development

The backing infrastructure is more generic and allows for passing of messages using other Streams and Sinks (for example, a TCP socket), not just stdin and stdout. However, due to
all this async stuff being new and at the moment super unstable in Rust, this crate is not currently ready or stable enough to expose the underlying API.
Feel free to [browse the source](https://github.com/vampirc/vampirc-io), though.

## Changelog 

### 0.3.0

* No longer require nightly rust, since async support has been added to stable.
* Upgraded `async-std` to 1.5 and `futures` to 0.3.
* Fixed `vampirc-uci` to 0.9.

### 0.2.1

* Support for async-std 0.99.7.