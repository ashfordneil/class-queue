use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use harsh::{Harsh, HarshBuilder};

use rand::{OsRng, Rng};

use toml;

#[derive(Deserialize)]
pub struct Config {
    pub root_dir: PathBuf,
    pub bcrypt_password: String,
    #[serde(skip_deserializing, default = "build_harsh")]
    pub hasher: Harsh,
}

impl Config {
    pub fn new() -> Self {
        let mut file = File::open("config.toml").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        toml::from_slice(&buf).unwrap()
    }
}

fn build_harsh() -> Harsh {
    let mut salt = [0; 64];
    OsRng::new().unwrap().fill_bytes(&mut salt[..]);

    HarshBuilder::new()
        .salt(&salt[..])
        .length(20)
        .init()
        .unwrap()
}
