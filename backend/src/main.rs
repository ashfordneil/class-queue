#[macro_use]
extern crate log;

extern crate simple_logger;

extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate toml;

extern crate mio;

extern crate rustls;

extern crate tungstenite;

#[macro_use]
mod result;

mod config;
use config::Config;

fn main() {
    simple_logger::init().unwrap();
    let config = Config::new("./config.toml");
}
