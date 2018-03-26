#[macro_use]
extern crate log;

extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate toml;

extern crate mio;

extern crate rustls;

extern crate tungstenite;

#[macro_use]
mod result;
mod error;
pub use error::{Error, Result};

mod config;
pub use config::Config;

mod acceptor;
pub use acceptor::Acceptor;

mod connection;
pub use connection::Connection;
