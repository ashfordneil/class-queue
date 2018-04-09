extern crate bcrypt;
extern crate bytes;
extern crate either;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate futures;
extern crate harsh;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate rpassword;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio;
extern crate unicase;
extern crate websocket;

mod application;
use application::State;

mod mpmc;
use mpmc::Mpmc;

mod patch;
use patch::TcpStream;

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use either::Either;

use futures::{Future, IntoFuture, Sink, Stream};
use futures::future::{self, Executor, Loop};

use hyper::{Method, Response, StatusCode};
use hyper::header::Allow;

use tokio::net::TcpListener;

use tokio_io::io;

use tokio_core::reactor::Core;

use websocket::async::server::IntoWs;
use websocket::message::OwnedMessage;
use websocket::client::async::Client;
use websocket::server::upgrade::Request;

fn dev_null<T>(_: T) {}

fn handle_request(root: &Path, req: &Request) -> Response<Vec<u8>> {
    let (ref method, ref request) = req.subject;
    if method.to_string() == "GET" {
        let request = request.to_string();
        let request = if request == "/" { "index.html".into() } else { request };
        let path = root.components().chain(Path::new(&request)
            .components()
            .filter(|component| match component {
                &Component::Normal(_) => true,
                _ => false,
            })
            )
            .collect::<PathBuf>();

        info!("Static request to {:?}", path);
        match File::open(&path).and_then(|file| file.bytes().collect::<Result<Vec<u8>, _>>()) {
            Ok(data) => Response::new().with_body(data),
            Err(_) => Response::new().with_status(StatusCode::NotFound),
        }
    } else {
        Response::new()
            .with_status(StatusCode::MethodNotAllowed)
            .with_header(Allow(vec![Method::Get]))
    }
}

fn http_serialize(res: Response<Vec<u8>>) -> Vec<u8> {
    let mut output = Vec::new();

    write!(output, "{} {}\r\n", res.version(), res.status()).unwrap();
    write!(output, "{}\r\n", res.headers()).unwrap();
    if let Some(body) = res.body_ref() {
        output.extend_from_slice(&body[..]);
    }
    output
}

fn main() {
    env_logger::init();
    let state = State::new();
    let root = application::root_dir();

    info!("Warming up");
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let server = TcpListener::bind(&"0.0.0.0:8080".parse().unwrap()).unwrap();
    let queue = Mpmc::new();
    info!("Ready to roll");

    let future = server
        .incoming()
        .map_err(dev_null)
        // accept the websocket connections, send HTTP to the non-websocket connections
        .and_then(|incomming| TcpStream(incomming).into_ws().then(|incomming| {
            let future: Box<Future<Item = Option<Client<_>>, Error = ()>> =
                match incomming {
                    Ok(upgrade) => {
                        Box::new(upgrade.accept().map(|(client, _headers)| Some(client)).map_err(dev_null))
                    }
                    Err((stream, req, _, error)) => {
                        if let Some(req) = req {
                            let response = handle_request(&root, &req);
                            let output = http_serialize(response);
                            handle.execute(io::write_all(stream, output).map(dev_null).map_err(dev_null))
                                .unwrap();
                        } else {
                            warn!("HTTP / Websocket Error: {}", error);
                        }
                        Box::new(future::ok(None))
                    }
                };
            future
        }))
        .filter_map(|client| client)
        // filter out all non-text messages
        .for_each(|client| {
            let (sink, client_stream) = client.split();

            let client_stream = client_stream.map(Either::Left).map_err(dev_null);
            let queue_stream = queue.stream().unwrap().map(Either::Right).map_err(dev_null);
            let stream = client_stream.select(queue_stream).into_future();

            let first_message = state.connect();

            sink.send(first_message).map_err(dev_null)
                .join(stream.into_future().map_err(dev_null))
                .map(|(sink, (message, stream))| (sink, stream, message))
                .and_then(|res| future::loop_fn(res, |(mut sink, stream, message)| {
                    // first, look at the message that we just got
                    let do_continue = (|| {
                        match message {
                            Some(Either::Left(ws_msg)) => {
                                let inner = match ws_msg {
                                    OwnedMessage::Text(ref text) => serde_json::from_str(text).map_err(dev_null)?,
                                    OwnedMessage::Binary(ref bytes) => serde_json::from_slice(bytes).map_err(dev_null)?,
                                    OwnedMessage::Ping(data) => {
                                        sink.start_send(OwnedMessage::Pong(data)).map_err(dev_null)?;
                                        return Ok(());
                                    }
                                    OwnedMessage::Pong(_) => return Ok(()),
                                    OwnedMessage::Close(_) => {
                                        sink.close().map_err(dev_null)?;
                                        return Err(());
                                    }
                                };

                                let (reply, internal) = state.from_client(inner);
                                if let Some(reply) = reply {
                                    sink.start_send(reply).map_err(dev_null)?;
                                }

                                if let Some(internal) = internal {
                                    queue.send(internal).unwrap();
                                }

                                Ok(())
                            },
                            Some(Either::Right(queue_msg)) => {
                                let msg: application::ServerMessage = queue_msg.into();
                                let output = OwnedMessage::Text(serde_json::to_string(&msg).unwrap());
                                sink.start_send(output).map_err(dev_null)?;
                                Ok(())
                            }
                            None => return Err(()),
                        }
                    })().is_ok();

                    sink.flush().map_err(dev_null)
                        .join(stream.into_future().map_err(dev_null))
                        .map(move |(sink, (message, stream))| if do_continue {
                            Loop::Continue((sink, stream, message))
                        } else {
                            Loop::Break(())
                        })
                }))
        });

    core.run(future).unwrap()
}
