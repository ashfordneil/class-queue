#[macro_use]
extern crate log;

extern crate simple_logger;

#[macro_use]
extern crate class_queue;
use class_queue::{Acceptor, Config, Connection, Result};

extern crate mio;
use mio::{Events, Poll, PollOpt, Ready, Token};

extern crate slab;
use slab::Slab;

fn fake_main() -> Result<()> {
    simple_logger::init().unwrap();
    let config = Config::new("config.toml");

    let mut events = Events::with_capacity(1024);
    let poll = Poll::new()?;
    let mut token_map = Slab::new();

    let acceptor = Acceptor::new(&config)?;
    poll.register(&acceptor, Token(0), Ready::readable(), PollOpt::edge())?;

    loop {
        poll.poll(&mut events, None)?;

        for event in &events {
            match event.token() {
                Token(0) => match acceptor.handle() {
                    Ok((session, stream)) => {
                        let entry = token_map.vacant_entry();
                        let raw_token = entry.key();
                        poll.register(
                            &stream,
                            Token(raw_token + 1),
                            Ready::readable() | Ready::writable(),
                            PollOpt::edge(),
                        )?;
                        let conn = match Connection::new(session, stream) {
                            Ok(conn) => conn,
                            Err(_) => {
                                continue;
                            }
                        };
                        entry.insert(conn);
                    }
                    Err(_) => (),
                },
                Token(n) => {
                    match token_map.get_mut(n - 1).unwrap().handle() {
                        Ok(Some(msg)) => info!("Received {:?} from client {}", msg, n - 1),
                        Ok(None) => (),
                        Err(_) => {
                            token_map.remove(n - 1);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    unwrap!(fake_main())
}
