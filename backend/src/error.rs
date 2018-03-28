//! Error handling within the class queue.
//!
//! All errors boil down to a single error enum that tracks its cause, and forwards its display
//! implementation with some context.
//!
//! As well as this enum, a macro throw has been defined - similar to try! - that logs errors at
//! the warn level and then propagates them up.

use std::io;
use std::result;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: io::Error) {
            display("IO error: {}", err)
            cause(err)
            from()
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

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
