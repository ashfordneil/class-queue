use mio::net::TcpStream;

use rustls::{ServerSession, Session};

use std::io::{Result, Read, Write};

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
    fn complete_prior_io(&mut self) -> Result<()> {
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
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
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
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.complete_prior_io()?;

        let &mut Stream {
            ref mut sess,
            ref mut sock,
        } = self;

        let len = sess.write(buf)?;
        sess.complete_io(sock)?;
        Ok(len)
    }

    fn flush(&mut self) -> Result<()> {
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
