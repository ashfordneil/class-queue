use mio::{Events, Poll, PollOpt, Ready, Token};

use slab::Slab;

use std::iter::Iterator;

use acceptor::Acceptor;
use config::Config;
use connection::Connection;
use error::Result;

/// Internal state of the server as a whole.
pub struct Server<'a> {
    events: Events,
    poll: Poll,
    acceptor: Acceptor,
    connections: Slab<Connection<'a>>,
    config: &'a Config,
}

impl<'a> Server<'a> {
    /// Create a new server.
    pub fn new(config: &'a Config) -> Result<Self> {
        let events = Events::with_capacity(config.events_capacity);
        let poll = Poll::new()?;
        let acceptor = Acceptor::new(config)?;
        let connections = Slab::new();

        poll.register(&acceptor, Token(0), Ready::readable(), PollOpt::level())?;

        Ok(Server {
            events,
            poll,
            acceptor,
            connections,
            config,
        })
    }

    /// Poll for events. This is a blocking call. Will return a vector of all the events that have
    /// occured.
    pub fn poll(&mut self) -> Result<Vec<Event>> {
        let &mut Server {
            ref poll,
            ref mut events,
            ..
        } = self;
        poll.poll(events, None)?;

        let output = events
            .iter()
            .map(|event| match event.token() {
                Token(0) => Event::Acceptor,
                Token(n) => Event::Connection(n - 1),
            })
            .collect();

        events.clear();

        Ok(output)
    }

    /// Attempt to accept a new connection, and begin its handshake.
    pub fn accept(&mut self) -> Result<()> {
        let (session, stream) = self.acceptor.handle()?;
        let entry = self.connections.vacant_entry();
        self.poll.register(
            &stream,
            Token(entry.key() + 1),
            Ready::readable() | Ready::writable(),
            PollOpt::edge(),
        )?;
        let conn = Connection::new(self.config, session, stream);
        entry.insert(conn);

        Ok(())

    }

    /// Get an individual connection handle out of the server. Should be called when an event is
    /// registered on that handle.
    pub fn get_connection(&mut self, token: usize) -> Option<&mut Connection<'a>> {
        self.connections.get_mut(token)
    }

    pub fn remove_connection(&mut self, token: usize) {
        self.connections.remove(token);
    }
}

/// Possible events that can happen to the server. Only stores which socket the event occured for,
/// rather than the entire event struct, because the other information cannot really be used.
pub enum Event {
    /// The acceptor of the server has had an event register on it.
    Acceptor,
    /// One of the connections, accessible via the token value, has had an event register on it.
    Connection(usize),
}
