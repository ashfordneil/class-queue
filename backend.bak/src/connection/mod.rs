use mio::net::TcpStream;

use rustls::{ServerSession, Session};

use httparse::{self, Request};

use tungstenite::{self, Error, WebSocket};
use tungstenite::handshake::{HandshakeError, MidHandshake};
use tungstenite::handshake::server::{NoCallback, ServerHandshake};

use std::io::{self, Read, Write};
use std::mem;
use std::path::Path;

use config::Config;
use error::Result;

mod http;

mod stream;
pub use self::stream::Stream;


/// A connection to a client.
pub struct Connection<'a> {
    state: ConnectionState,
    config: ConnectionConfig<'a>,
}

enum ConnectionState {
    /// For when the socket is currently reading a http request.
    ReadingHttp { buffer: Vec<u8>, stream: Stream },
    /// For when the socket is currently writing a http resonse.
    WritingHttp {
        buffer: Vec<u8>,
        stream: Stream,
        become_websocket: bool,
    },
    /// For when the socket is working with the websocket protocol.
    WebSocket(WebSocket<Stream>),
}

enum ConnectionStateChange {
    StartWriting(Vec<u8>),
    FinishWriting,
    BecomeWebsocket,
}

struct ConnectionConfig<'a> {
    max_headers: usize,
    buffer_size: usize,
    static_dir: &'a Path,
}

impl<'a> Connection<'a> {
    /// Create a new connection, using a provided configuration, a tls session and a socket.
    pub fn new(config: &'a Config, session: ServerSession, stream: TcpStream) -> Self {
        let stream = Stream::new(session, stream);
        let state = ConnectionState::ReadingHttp {
            buffer: Vec::new(),
            stream,
        };
        let &Config {
            max_headers,
            buffer_size,
            ref static_dir,
            ..
        } = config;
        let config = ConnectionConfig {
            max_headers,
            buffer_size,
            static_dir,
        };
        Connection { state, config }
    }

    /// Handle an IO event
    pub fn handle(&mut self) -> Result<()> {
        let &mut Connection { ref mut state, ref config } = self;
        let state_change = state.handle(config)?;
        if let Some(state_change) = state_change {
            // move the old state out
            let old_state = mem::replace(state, unsafe { mem::uninitialized() });

            // calculate the new state
            let new_state = match (old_state, state_change) {
            };

            // put the new state in
            mem::replace(state, new_state);
        }

        Ok(())
    }
}

impl ConnectionState {
    /// Figure out what to do with a HTTP request
    fn handle_request<'headers, 'buf: 'headers>(
        config: &ConnectionConfig,
        req: Request<'headers, 'buf>,
    ) -> ConnectionStateChange {
        if req.method.unwrap() != "GET" {
            return ConnectionStateChange::StartWriting(http::invalid_method());
        }
        unimplemented!()
    }

    /// Handle an IO event while in the reading state
    fn handle_reading(
        config: &ConnectionConfig,
        buffer: &mut Vec<u8>,
        stream: &mut Stream,
    ) -> Result<Option<ConnectionStateChange>> {
        let old_len = buffer.len();

        // make space (with uninitialized memory) for the buffer to read into
        buffer.reserve(config.buffer_size);
        unsafe { buffer.set_len(old_len + config.buffer_size) };

        let read_result = match stream.read(&mut buffer[old_len..]) {
            Ok(size) => {
                // gate off the uninitialized memory that was not read into
                let new_len = old_len + size;
                unsafe { buffer.set_len(new_len) };

                let mut headers = vec![httparse::EMPTY_HEADER; config.max_headers];
                let mut req = Request::new(&mut headers);
                req.parse(&buffer[..])?;

                if &buffer[(new_len - 4)..] == b"\r\n\r\n" {
                    // request is completely parsed
                    Ok(Some(Self::handle_request(config, req)))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                // gate off the uninitialized memory that was not read into
                unsafe { buffer.set_len(old_len) };

                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }?;

        Ok(read_result)
    }

    /// Handle an IO event
    pub fn handle(&mut self, config: &ConnectionConfig) -> Result<Option<ConnectionStateChange>> {
        match self {
            &mut ConnectionState::ReadingHttp {
                ref mut buffer,
                ref mut stream,
            } => Self::handle_reading(config, buffer, stream),
            &mut ConnectionState::WritingHttp {
                ref mut buffer,
                ref mut stream,
                become_websocket,
            } => unimplemented!(),
            &mut ConnectionState::WebSocket(ref mut ws) => unimplemented!(),
        }
    }
}
