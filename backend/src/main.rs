#[macro_use]
extern crate log;

extern crate simple_logger;

#[macro_use]
extern crate class_queue;
use class_queue::acceptor::Acceptor;
use class_queue::config::Config;

fn main() {
    simple_logger::init().unwrap();
    let config = Config::new("config.toml");

    let acceptor = unwrap!(Acceptor::new(&config));
}
