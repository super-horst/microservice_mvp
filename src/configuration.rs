use std::env;
use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub port: u32,
    pub targets: Vec<u32>,
    pub loglevel: String,
}

pub fn load_config() -> Config {
    let config_path = env::var("CONFIG").unwrap();
    return confy::load_path(config_path).unwrap();
}
