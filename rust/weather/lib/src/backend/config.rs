//! Utilities to load application configurations from `TOML` files at runtime.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};

macro_rules! err {
    ($reason:expr) => {
        Err(Error::from(format!("Config: {}", $reason)))
    };
}
pub(self) use err;

const DEFAULT_FILENAME: &'static str = "weather.toml";

#[derive(Debug)]
pub struct Config {
    pub weather_data: weather_data::Properties,
    pub visual_crossing: visual_crossing::Properties,
    pub us_cities: us_cities::Properties,
}
impl Config {
    pub fn new(optional_path: Option<PathBuf>) -> Result<Config> {
        config_file::load(optional_path)
    }
}
impl TryFrom<&str> for Config {
    type Error = Error;
    /// Attempt to load the configuration from a string.
    fn try_from(config_str: &str) -> std::result::Result<Self, Self::Error> {
        Ok(Self::from(config_file::load_str(config_str)?))
    }
}

mod config_file {
    //! The configuration file manager.
    use super::*;
    use std::{fs::File, io::prelude::*};
    use toml;

    /// Try to get the configuration from the file pathname. If it was not provided
    /// try the default filename. If the default filename does not exist use defaults.
    pub fn load(optional_path: Option<PathBuf>) -> Result<Config> {
        match optional_path {
            Some(path) => match (path.exists(), path.is_file()) {
                (true, true) => Ok(Config::from(load_path(&path)?)),
                (true, false) => err!("Configuration name is not a file."),
                _ => err!("Configuration name not found."),
            }
            None => {
                // try loading the default filename
                let path = PathBuf::from(DEFAULT_FILENAME);
                match (path.exists(), path.is_file()) {
                    (true, true) => Ok(Config::from(load_path(&path)?)),
                    (true, false) => err!(format!("{} is not a file.", DEFAULT_FILENAME)),
                    _ => {
                        log::info!("Did not find a configuration file, using defaults");
                        Ok(Config::from(ConfigDocument::default()))
                    }
                }
            }
        }
    }

    /// The structure that holds the weather configuration document.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct ConfigDocument {
        #[serde(rename = "weather-data")]
        pub weather_data: Option<weather_data::Document>,
        #[serde(rename = "visual-crossing")]
        pub visual_crossing: Option<visual_crossing::Document>,
        #[serde(rename = "us-cities")]
        pub us_cities: Option<us_cities::Document>,
    }
    impl From<ConfigDocument> for Config {
        /// Create the configuration from the configuration document instance.
        fn from(config_document: ConfigDocument) -> Self {
            Config {
                weather_data: weather_data::Properties::from(config_document.weather_data),
                visual_crossing: visual_crossing::Properties::from(config_document.visual_crossing),
                us_cities: us_cities::Properties::from(config_document.us_cities),
            }
        }
    }

    /// Attempts to load the configuration from a file.
    fn load_path(path: &Path) -> Result<ConfigDocument> {
        match File::open(path) {
            Ok(mut file) => {
                let mut contents = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_) => load_str(&contents),
                    Err(err) => {
                        err!(format!("Could not read '{}' contents ({})", path.display(), err))
                    }
                }
            }
            Err(err) => {
                err!(format!("Could not open '{}' ({}).", path.display(), err))
            }
        }
    }

    /// Attempts to load the configuration from a string.
    pub fn load_str(config: &str) -> Result<ConfigDocument> {
        match toml::from_str::<ConfigDocument>(config) {
            Ok(config) => Ok(config),
            Err(err) => Err(Error::from(format!("Could not load the configuration ({}).", err))),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        macro_rules! as_ref {
            ($value:expr) => {
                $value.as_ref().unwrap()
            };
        }
        macro_rules! some {
            ($value:literal) => {
                Some($value.to_string())
            };
        }

        #[test]
        fn load() {
            let config = r#"
                [weather-data]
                directory = "directory/name"

                [visual-crossing]
                endpoint = "http://end/point"
                api-key = "api-key"

                [us-cities]
                filename = "filename.csv"
                "#;
            let testcase = load_str(config).unwrap();
            assert_eq!(as_ref!(testcase.weather_data).directory, some!("directory/name"));
            assert_eq!(as_ref!(testcase.visual_crossing).endpoint, some!("http://end/point"));
            assert_eq!(as_ref!(testcase.visual_crossing).api_key, some!("api-key"));
            assert_eq!(as_ref!(testcase.us_cities).filename, some!("filename.csv"));
            let config = r#"
                [weather-data]
                directory = "directory/name"
                "#;
            let testcase = load_str(config).unwrap();
            assert!(testcase.weather_data.is_some());
            assert!(testcase.visual_crossing.is_none());
            assert!(testcase.us_cities.is_none());
            let config = r#"
                [visual-crossing]
                endpoint = "http://end/point"
                api-key = "api-key"
                "#;
            let testcase = load_str(config).unwrap();
            assert!(testcase.weather_data.is_none());
            assert!(testcase.visual_crossing.is_some());
            assert!(testcase.us_cities.is_none());
            let config = r#"
                [us-cities]
                filename = "filename.csv"
                "#;
            let testcase = load_str(config).unwrap();
            assert!(testcase.weather_data.is_none());
            assert!(testcase.visual_crossing.is_none());
            assert!(testcase.us_cities.is_some());
        }

        #[test]
        fn env() {
            // isolate env setting here to avoid threaded test failures
            env::remove_var(weather_data::ENV_DIRNAME);
            env::remove_var(visual_crossing::ENV_KEY);
            env::remove_var(us_cities::ENV_FILENAME);
            let testcase = Config::from(ConfigDocument::default());
            log::debug!("{:#?}", testcase);
            assert_eq!(testcase.weather_data.directory, weather_data::DEFAULT_DIRNAME);
            assert_eq!(testcase.visual_crossing.endpoint, visual_crossing::DEFAULT_URI);
            assert_eq!(testcase.visual_crossing.api_key, visual_crossing::DEFAULT_KEY);
            assert_eq!(testcase.us_cities.filename, us_cities::DEFAULT_FILENAME);
            //
            env::remove_var(weather_data::ENV_DIRNAME);
            env::remove_var(visual_crossing::ENV_KEY);
            env::remove_var(us_cities::ENV_FILENAME);
            let dirname = "dirname";
            let key = "A key";
            let filename = "filename";
            env::set_var(weather_data::ENV_DIRNAME, dirname);
            env::set_var(visual_crossing::ENV_KEY, key);
            env::set_var(us_cities::ENV_FILENAME, filename);
            let testcase = Config::from(ConfigDocument::default());
            assert_eq!(testcase.weather_data.directory, dirname);
            assert_eq!(testcase.visual_crossing.endpoint, visual_crossing::DEFAULT_URI);
            assert_eq!(testcase.visual_crossing.api_key, key);
            assert_eq!(testcase.us_cities.filename, filename);
            env::remove_var(weather_data::ENV_DIRNAME);
            env::remove_var(visual_crossing::ENV_KEY);
            env::remove_var(us_cities::ENV_FILENAME);
        }
    }
}

mod weather_data {
    //! The weather data configuration table.
    use super::*;

    pub const ENV_DIRNAME: &'static str = "WEATHER_DATA";
    pub const DEFAULT_DIRNAME: &'static str = "weather_data";

    #[derive(Debug)]
    pub struct Properties {
        pub directory: String,
    }
    impl From<Option<Document>> for Properties {
        /// Convert the document into the configuration table.
        fn from(value: Option<Document>) -> Self {
            match value {
                Some(dict) => {
                    let directory = dict.directory.unwrap_or_else(default_dirname);
                    Properties { directory }
                }
                None => Properties { directory: default_dirname() },
            }
        }
    }

    /// The configuration that can be serialized and deserialized.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Document {
        pub directory: Option<String>,
    }

    /// Gets the default API key from the process environment if [ENV_DIRNAME] is defined.
    fn default_dirname() -> String {
        env::var(ENV_DIRNAME).unwrap_or_else(|_| DEFAULT_DIRNAME.to_string())
    }
}

mod visual_crossing {
    //! The Visual Crossing configuration data.
    use super::*;

    pub const ENV_KEY: &'static str = "VISUAL_CROSSING_KEY";
    pub const DEFAULT_KEY: &'static str = "API_KEY";
    pub const DEFAULT_URI: &'static str =
        "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline";

    #[derive(Debug)]
    pub struct Properties {
        pub endpoint: String,
        pub api_key: String,
    }
    impl From<Option<Document>> for Properties {
        /// Convert the document into the configuration table.
        fn from(value: Option<Document>) -> Self {
            match value {
                Some(dict) => {
                    let endpoint = dict.endpoint.unwrap_or(DEFAULT_URI.to_string());
                    let api_key = dict.api_key.unwrap_or_else(default_api_key);
                    Properties { endpoint, api_key }
                }
                None => Properties { endpoint: DEFAULT_URI.to_string(), api_key: default_api_key() },
            }
        }
    }

    /// The Visual Crossing configuration options.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Document {
        /// The API end-point.
        pub endpoint: Option<String>,
        /// The API key token.
        #[serde(rename = "api-key")]
        pub api_key: Option<String>,
    }

    /// Gets the default API key from the process environment if [ENV_KEY] is defined.
    fn default_api_key() -> String {
        env::var(ENV_KEY).unwrap_or_else(|_| DEFAULT_KEY.to_string())
    }
}

mod us_cities {
    //! The Visual Crossing configuration data.
    use super::*;

    pub const ENV_FILENAME: &'static str = "USCITIES_FILENAME";
    pub const DEFAULT_FILENAME: &'static str = "uscities.csv";

    #[derive(Debug)]
    pub struct Properties {
        pub filename: String,
    }
    impl From<Option<Document>> for Properties {
        /// Convert the document into the configuration table.
        fn from(value: Option<Document>) -> Self {
            match value {
                Some(dict) => {
                    let filename = dict.filename.unwrap_or_else(default_filename);
                    Properties { filename }
                }
                None => Properties { filename: default_filename() },
            }
        }
    }

    /// The US Cities configuration options.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Document {
        pub filename: Option<String>,
    }

    /// Gets the default filename from the process environment if [ENV_FILENAME] is defined.
    fn default_filename() -> String {
        env::var(ENV_FILENAME).unwrap_or_else(|_| DEFAULT_FILENAME.to_string())
    }
}
