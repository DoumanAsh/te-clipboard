use serde_derive::{Deserialize};
use serde::{Deserialize};
use regex::Regex;

use std::fs::File;
use std::borrow::Cow;
use std::io::Read;

pub fn deserialize_to_regex<'de, D: serde::de::Deserializer<'de>>(re_str: D) -> Result<Regex, D::Error> {
    let re_str: Cow<'de, str> = Cow::deserialize(re_str)?;

    Ok(Regex::new(&re_str).map_err(serde::de::Error::custom)?)
}

#[derive(Deserialize, Debug)]
pub struct Replace {
    #[serde(deserialize_with="deserialize_to_regex")]
    pub pattern: regex::Regex,
    pub replacement: String,
    #[serde(default)]
    pub limit: usize
}

#[derive(Deserialize, Debug, Default)]
///Configuration of application
pub struct DeConfig {
    //#[serde(default)]
    //pub text_repetitions: bool,
    #[serde(default)]
    pub replace: Option<Vec<Replace>>
}

#[derive(Debug, Default)]
///Configuration of application
pub struct Config {
    //pub text_repetitions: bool,
    pub replace: Vec<Replace>
}

impl Into<Config> for DeConfig {
    fn into(self) -> Config {
        Config {
            replace: self.replace.unwrap_or_else(|| Vec::with_capacity(0)),
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => return Config::default(),
        };

        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Ok(_) => (),
            Err(_) => return Config::default(),
        }

        match toml::from_str::<DeConfig>(&buffer) {
            Ok(config) => config.into(),
            Err(_) => Config::default(),
        }
    }
}
