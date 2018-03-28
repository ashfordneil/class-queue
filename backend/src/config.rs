use error::Result;

use std::fs::File;
use std::io::{BufReader, Read};
use std::result;
use std::path::Path;

use rustls::{Certificate, PrivateKey};
use rustls::internal::pemfile;

use serde::de::{Deserialize, Deserializer, Error};

use toml;

/// Configuration files within the class queue.
///
/// All configurations are loaded from a single toml file, which describes everything needed for
/// the program to run.
#[derive(Debug, Deserialize)]
pub struct Config {
    // TLS parameters
    #[serde(deserialize_with = "read_cert")] certificate: Vec<Certificate>,
    #[serde(deserialize_with = "read_priv_key")] priv_key: PrivateKey,

    // Socket parameters
    pub port: u16,
}

impl Config {
    /// Read from a specified toml file, and create a new config object.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = throw!(File::open(path));
        let mut buffer = Vec::new();
        throw!(file.read_to_end(&mut buffer));
        let output = throw!(toml::from_slice(&buffer[..]));
        Ok(output)
    }
}

/// Helper function for deserializing the certificate chain. Reads a file path from the toml file,
/// and then separately opens the file pointed to by the path and parses it.
fn read_cert<'de, D>(de: D) -> result::Result<Vec<Certificate>, D::Error>
where
    D: Deserializer<'de>,
{
    let file_name = <&Path as Deserialize<'de>>::deserialize(de)?;
    info!("Loading certificates from {:?}", file_name);
    let file = File::open(file_name).map_err(D::Error::custom)?;
    let mut wrapper = BufReader::new(file);
    pemfile::certs(&mut wrapper).map_err(|_| D::Error::custom("could not load certs"))
}

/// Helper function for deserializing the private key. Reads a file path from the toml file, and
/// then separately opens the file pointer to by the path, and parses it as if it is an RSA key.
fn read_priv_key<'de, D>(de: D) -> result::Result<PrivateKey, D::Error>
where
    D: Deserializer<'de>,
{
    let file_name = <&Path as Deserialize<'de>>::deserialize(de)?;
    info!("Loading private key from {:?}", file_name);
    let file = File::open(file_name).map_err(D::Error::custom)?;
    let mut wrapper = BufReader::new(file);
    let keys = pemfile::rsa_private_keys(&mut wrapper)
        .map_err(|_| D::Error::custom("could not load certs"))?;

    keys.into_iter()
        .next()
        .ok_or(D::Error::custom("No RSA keys found"))
}
