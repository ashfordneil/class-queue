use std::io;
use std::result;

use toml::de;

quick_error! {
    /// Error handling within the class queue.
    ///
    /// Al errors boil down to this single error enum, that tracks error causes and forwards
    /// display implementations (with context).
    #[derive(Debug)]
    pub enum Error {
        Io(err: io::Error) {
            display("IO error: {}", err)
            cause(err)
            from()
        }
        Config(err: de::Error) {
            display("Config file error: {}", err)
            cause(err)
            from()
        }
    }
}

/// A new result type for all functions within the class queue that have a chance of failure.
pub type Result<T> = result::Result<T, Error>;

/// A macro that is similar to try!, except it logs errors (at a warn level) before propagating
/// them up to higher levels.
#[macro_export]
macro_rules! throw {
    ($result:expr) =>  {
        match $result {
            Ok(t) => t,
            Err(e) => {
                let e = e.into();
                warn!("{}", e);
                return Err(e);
            },
        }
    }
}
