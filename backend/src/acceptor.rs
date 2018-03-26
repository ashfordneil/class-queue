use mio::{Evented, Poll, PollOpt, Ready, Token};
use mio::net::{TcpListener, TcpStream};

use rustls::{ServerConfig, ServerSession};

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use error::Result;
use config::Config;

/// Internal state of the acceptor itself.
pub struct Acceptor {
    config: Arc<ServerConfig>,
    sock: TcpListener,
}

impl Acceptor {
    /// Creates a new acceptor, from the config results. Will bind to the port listed, so may fail.
    pub fn new(config: &Config) -> Result<Self> {
        let tls_config = Arc::new(config.tls_config());
        let sock = TcpListener::bind(&SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            config.port,
        ))?;
        Ok(Acceptor {
            config: tls_config,
            sock,
        })
    }

    /// When an event is ready that is associated with this acceptor, handle it. Produces the data
    /// necessary to create a new Connection, but that data needs to be stored separately to the
    /// connection itself.
    pub fn handle(&self) -> Result<(ServerSession, TcpStream)> {
        let stream = match self.sock.accept() {
            Ok((stream, _)) => Ok(stream),
            Err(e) => {
                warn!("Could not accept socket: {}", e);
                Err(e)
            }
        }?;
        let session = ServerSession::new(&self.config);
        Ok((session, stream))
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
