#[macro_use]
extern crate class_queue;
#[macro_use]
extern crate log;
extern crate env_logger;

use class_queue::{Config, Result};

use std::process;

// runs the application logic of the program, forwarding errors up to main
fn fake_main() -> Result<()> {
    let config = Config::new("config.toml")?;

    Ok(())
}

// initialize logger, then call the fake_main and handle any errors it throws
fn main() {
    env_logger::init();

    match fake_main() {
        Ok(()) => (),
        Err(e) => {
            error!("Fatal Error: {}", e);
            process::exit(1)
        }
    }
}
