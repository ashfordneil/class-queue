use error::Result;
use config::Config;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use mio::{Evented, Poll, PollOpt, Ready, Token};
use mio::net::{TcpListener, TcpStream};

/// Internal state of the socket acceptor.
pub struct Acceptor {
    sock: TcpListener,
}

impl Acceptor {
    /// Creates a new acceptor, from the config results. Will bind to the port listed, so may fail.
    pub fn new(config: &Config) -> Result<Self> {
        let sock = throw!(TcpListener::bind(&SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                config.port,
        )));
        Ok(Acceptor {
            sock
        })
    }
}

impl Evented for Acceptor {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.sock.register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.sock.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.sock.deregister(poll)
    }
}
