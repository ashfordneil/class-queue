use serde::de::{Deserialize, Deserializer, Error};

use toml;

use rustls::{Certificate, PrivateKey};
use rustls::internal::pemfile;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "read_cert")] certificate: Vec<Certificate>,
    #[serde(deserialize_with = "read_priv_key")] priv_key: PrivateKey,
    pub port: u16,
}

impl Config {
    /// Read from a config file, create a new config. Will exit the process on error.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut file = unwrap!(File::open(path));
        let mut buffer = Vec::new();
        unwrap!(file.read_to_end(&mut buffer));
        unwrap!(toml::from_slice(&buffer[..]))
    }
}

fn read_cert<'de, D>(de: D) -> Result<Vec<Certificate>, D::Error>
where
    D: Deserializer<'de>,
{
    let file_name = <&Path as Deserialize<'de>>::deserialize(de)?;
    info!("Loading certificates from {:?}", file_name);
    let file = File::open(file_name).map_err(D::Error::custom)?;
    let mut wrapper = BufReader::new(file);
    pemfile::certs(&mut wrapper).map_err(|_| D::Error::custom("Could not load certs"))
}

fn read_priv_key<'de, D>(de: D) -> Result<PrivateKey, D::Error>
where
    D: Deserializer<'de>,
{
    let file_name = <&Path as Deserialize<'de>>::deserialize(de)?;
    info!("Loading private key from {:?}", file_name);
    let file = File::open(file_name).map_err(D::Error::custom)?;
    let mut wrapper = BufReader::new(file);
    let keys = pemfile::rsa_private_keys(&mut wrapper)
        .map_err(|_| D::Error::custom("Could not load pkcs8 private key"))?;

    keys.into_iter().next().ok_or(D::Error::custom("No pkcs8 keys found"))
}
