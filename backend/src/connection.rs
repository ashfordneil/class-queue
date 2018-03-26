use mio::net::TcpStream;

use rustls::{ServerSession, Session};

use tungstenite::{self, Error, Message, WebSocket};
use tungstenite::handshake::{HandshakeError, MidHandshake};
use tungstenite::handshake::server::{NoCallback, ServerHandshake};

use std::io::{self, Read, Write};
use std::mem;

use error::Result;

/// TLS wrapper around a writer + reader.
pub struct Stream {
    sess: ServerSession,
    sock: TcpStream,
}

impl Stream {
    /// Create a new stream.
    pub fn new(sess: ServerSession, sock: TcpStream) -> Self {
        Stream { sess, sock }
    }

    /// If we're handshaking, complete all the IO for that. If we have data to write, write it all.
    fn complete_prior_io(&mut self) -> io::Result<()> {
        let &mut Stream {
            ref mut sess,
            ref mut sock,
        } = self;
        if sess.is_handshaking() {
            sess.complete_io(sock)?;
        }

        if sess.wants_write() {
            sess.complete_io(sock)?;
        }

        Ok(())
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.complete_prior_io()?;

        let &mut Stream {
            ref mut sess,
            ref mut sock,
        } = self;

        if sess.wants_read() {
            sess.complete_io(sock)?;
        }

        sess.read(buf)
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.complete_prior_io()?;

        let &mut Stream {
            ref mut sess,
            ref mut sock,
        } = self;

        let len = sess.write(buf)?;
        sess.complete_io(sock)?;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.complete_prior_io()?;

        let &mut Stream {
            ref mut sess,
            ref mut sock,
        } = self;

        sess.flush()?;
        if sess.wants_write() {
            sess.complete_io(sock)?;
        }

        Ok(())
    }
}

/// A connection to a client.
pub enum Connection {
    /// The connection is currently midway through the websocket handshake.
    MidHandshake(MidHandshake<ServerHandshake<Stream, NoCallback>>),
    NormalOperation(WebSocket<Stream>),
}

impl Connection {
    /// Creates a new connection, from the information provided by an acceptor. Assumes that the
    /// socket provided is fresh, and has not had any data sent or received to / from it yet.
    pub fn new(session: ServerSession, stream: TcpStream) -> Result<Self> {
        let stream = Stream::new(session, stream);

        let output = match tungstenite::accept(stream) {
            Ok(complete) => Ok(Connection::NormalOperation(complete)),
            Err(HandshakeError::Interrupted(state)) => Ok(Connection::MidHandshake(state)),
            Err(HandshakeError::Failure(error)) => Err(error),
        }?;

        Ok(output)
    }

    pub fn handle(&mut self) -> Result<Option<Message>> {
        if let &mut Connection::NormalOperation(ref mut sock) = self {
            match sock.write_pending() {
                Ok(()) => Ok(()),
                Err(Error::Io(ref err)) if err.kind() == io::ErrorKind::WouldBlock => Ok(()),
                Err(e) => {
                    warn!("Websocket error: {}", e);
                    Err(e)
                },
            }?;
            let message = match sock.read_message() {
                Ok(message) => Ok(Some(message)),
                Err(Error::Io(ref err)) if err.kind() == io::ErrorKind::WouldBlock => Ok(None),
                Err(e) => {
                    warn!("Websocket error: {}", e);
                    Err(e)
                },
            }?;
            Ok(message)
        } else {
            let default = unsafe {
                mem::uninitialized::<Connection>()
            };
            let old = mem::replace(self, default);

            if let Connection::MidHandshake(state) = old {
                let output = match state.handshake() {
                    Ok(complete) => Ok(Connection::NormalOperation(complete)),
                    Err(HandshakeError::Interrupted(state)) => Ok(Connection::MidHandshake(state)),
                    Err(HandshakeError::Failure(error)) => Err(error),
                }?;
                *self = output;
                Ok(None)
            } else {
                unreachable!()
            }
        }
    }
}
