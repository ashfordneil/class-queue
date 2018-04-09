//! A patch around a certain trait implementation in order to get better errors.
use std::io::{self, ErrorKind};

use bytes::BytesMut;

use futures::{Future, Stream};

use tokio::net;
use tokio_io::AsyncRead;
use tokio_io::codec::FramedParts;

use unicase::UniCase;

use websocket::async::server::IntoWs;
use websocket::codec::http::HttpServerCodec;
use websocket::header::{Connection, ConnectionOption, Headers, ProtocolName, Upgrade, WebSocketKey, WebSocketVersion};
use websocket::server::upgrade::{HyperIntoWsError, Request, WsUpgrade};

pub struct TcpStream(pub net::TcpStream);

impl IntoWs for TcpStream
{
	type Stream = net::TcpStream;
	type Error = (net::TcpStream, Option<Request>, BytesMut, HyperIntoWsError);

	fn into_ws(self) -> Box<Future<Item = WsUpgrade<Self::Stream, BytesMut>, Error = Self::Error>> {
        let validate = |method, version, headers: &Headers| {
            if format!("{}", method) != "GET" {
                return Err(HyperIntoWsError::MethodNotGet);
            }

            let version = format!("{}", version);
            if version == "HTTP/0.9" || version == "HTTP/1.0" {
                return Err(HyperIntoWsError::UnsupportedHttpVersion);
            }

            if let Some(version) = headers.get::<WebSocketVersion>() {
                if version != &WebSocketVersion::WebSocket13 {
                    return Err(HyperIntoWsError::UnsupportedWebsocketVersion);
                }
            }

            if headers.get::<WebSocketKey>().is_none() {
                return Err(HyperIntoWsError::NoSecWsKeyHeader);
            }

            match headers.get() {
                Some(&Upgrade(ref upgrade)) => {
                    if upgrade.iter().all(|u| u.name != ProtocolName::WebSocket) {
                        return Err(HyperIntoWsError::NoWsUpgradeHeader);
                    }
                }
                None => return Err(HyperIntoWsError::NoUpgradeHeader),
            };

            fn check_connection_header(headers: &[ConnectionOption]) -> bool {
                for header in headers {
                    if let ConnectionOption::ConnectionHeader(ref h) = *header {
                        if UniCase::new(h as &str) == UniCase::new("upgrade") {
                            return true;
                        }
                    }
                }
                false
            }

                match headers.get() {
                    Some(&Connection(ref connection)) => {
                        if !check_connection_header(connection) {
                            return Err(HyperIntoWsError::NoWsConnectionHeader);
                        }
                    }
                    None => return Err(HyperIntoWsError::NoConnectionHeader),
                };

                Ok(())
            };

		let future = self.0.framed(HttpServerCodec)
          .into_future()
          .map_err(|(e, s)| {
              let FramedParts { inner, readbuf, .. } = s.into_parts();
              (inner, None, readbuf, e.into())
          })
          .and_then(move |(m, s)| {
              let FramedParts { inner, readbuf, .. } = s.into_parts();
              if let Some(msg) = m {
                  match validate(msg.subject.0.clone(), msg.version.clone(), &msg.headers) {
                      Ok(()) => Ok((msg, inner, readbuf)),
                      Err(e) => Err((inner, Some(msg), readbuf, e)),
                  }
              } else {
                  let err = HyperIntoWsError::Io(io::Error::new(
                      ErrorKind::ConnectionReset,
                  "Connection dropped before handshake could be read"));
                  Err((inner, None, readbuf, err))
              }
          })
          .map(|(m, stream, buffer)| {
              WsUpgrade {
                  headers: Headers::new(),
                  stream: stream,
                  request: m,
                  buffer: buffer,
              }
          });
		Box::new(future)
	}
}
