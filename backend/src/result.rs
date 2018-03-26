macro_rules! unwrap {
    ($result:expr) => {
        match $result {
            Ok(t) => t,
            Err(e) => {
                error!("Aborting due to {}", e);
                ::std::process::exit(1)
            }
        }
    }
}
