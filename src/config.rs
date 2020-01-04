use serde_derive::{Deserialize};
use serde::{Deserialize};
use regex::Regex;

use std::fs::File;
use std::borrow::Cow;
use std::io::Read;

use crate::rt;

const MISSING_CONFIG_ERROR: &str = "Config file is missing...

Please be sure that file te-clipboard.toml is present in directory with Textractor
";
const CANNOT_READ_CONFIG_ERROR: &str = "Cannot read config file...

Please be sure that file te-clipboard.toml is available for read to all users.
Or start Textractor with administrative rights.
";
const INVALID_CONFIG_ERROR: &str = "Config file is not valid TOML file...

Please be sure to check the format of file te-clipboard.toml
";

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
pub struct Settings {
    #[serde(default)]
    pub modify_original: bool
}

#[derive(Deserialize, Debug, Default)]
///Configuration of application
pub struct DeConfig {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub replace: Option<Vec<Replace>>
}

#[derive(Debug, Default)]
///Configuration of application
pub struct Config {
    pub settings: Settings,
    pub replace: Vec<Replace>
}

impl Into<Config> for DeConfig {
    fn into(self) -> Config {
        Config {
            settings: self.settings,
            replace: self.replace.unwrap_or_else(|| Vec::with_capacity(0)),
        }
    }
}

impl Config {
    #[inline(always)]
    pub fn get() -> &'static Self {
        use std::sync::Once;
        use core::mem::MaybeUninit;

        static mut CONFIG: MaybeUninit<Config> = MaybeUninit::uninit();
        static INIT: Once = Once::new();

        INIT.call_once(|| unsafe {
            core::ptr::write(CONFIG.as_mut_ptr(), Config::from_file("te-clipboard.toml"));
        });

        unsafe {
            &*CONFIG.as_ptr()
        }
    }

    pub fn from_file(path: &str) -> Self {
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => {
                rt::notify(MISSING_CONFIG_ERROR);
                return Config::default();
            }
        };

        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Ok(_) => (),
            Err(_) => {
                rt::notify(CANNOT_READ_CONFIG_ERROR);
                return Config::default();
            }
        }

        match toml::from_str::<DeConfig>(&buffer) {
            Ok(config) => config.into(),
            Err(_) => {
                rt::notify(INVALID_CONFIG_ERROR);
                Config::default()
            }
        }
    }
}
