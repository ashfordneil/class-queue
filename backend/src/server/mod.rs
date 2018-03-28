use config::Config;
use error::Result;

use std::iter::Iterator;

use mio::{Events, Poll, PollOpt, Ready, Token};

use slab::Slab;

mod acceptor;
use self::acceptor::Acceptor;

/// Internal state of the entire server.
pub struct Server<'a> {
    events: Events,
    poll: Poll,
    acceptor: Acceptor,
    config: &'a Config,
}

impl<'a> Server<'a> {
    /// Create a new server.
    pub fn new(config: &'a Config) -> Result<Self> {
        let events = Events::with_capacity(config.events_capacity);
        let poll = Poll::new()?;
        let acceptor = Acceptor::new(config)?;

        poll.register(&acceptor, Token(0), Ready::readable(), PollOpt::edge())?;

        Ok(Server {
            events,
            poll,
            acceptor,
            config
        })
    }
}
