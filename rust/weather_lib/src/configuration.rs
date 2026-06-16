//! Utilities to load application configurations from `TOML` files at runtime.

mod macros;
mod us_cities;
mod visual_crossing;
mod weather_data;

use serde::{Deserialize, Serialize};
use std::{env, path::Path};

/// The common properties error creator.
///
macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error::from(format!("Properties: {}", format!($($arg)*))))
    };
}

/// Get an environment variable or use the default if the variable is not found.
///
macro_rules! env_var_or {
    ($name:expr, $default:expr) => {
        match env::var($name) {
            Err(_) => $default.to_string(),
            Ok(mut value) => {
                value = value.trim().to_string();
                match value.is_empty() {
                    true => $default.to_string(),
                    false => value,
                }
            }
        }
    };
}
use env_var_or;

/// The configuration settings.
///
#[derive(Clone, Debug)]
pub struct Configuration {
    /// The weather data configuration properties.
    pub weather_data: weather_data::Properties,
    /// The Visual Crossing configuration properties.
    pub visual_crossing: visual_crossing::Properties,
    /// The US cities configuration properties.
    pub us_cities: us_cities::Properties,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            weather_data: weather_data::Properties::default(),
            visual_crossing: visual_crossing::Properties::default(),
            us_cities: us_cities::Properties::default(),
        }
    }
}
impl TryFrom<&Path> for Configuration {
    type Error = crate::Error;
    fn try_from(path: &Path) -> crate::Result<Self> {
        let document = file::Document::try_from(path)?;
        Ok(Configuration::from(document))
    }
}
impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let document = file::Document::from(self).with_defaults();
        write!(f, "{document}")
    }
}
impl Configuration {
    /// The default configuration filename.
    pub const DEFAULT_FILENAME: &'static str = "weather.toml";

    /// The environment variable holding the default configuration filename.
    pub const FILENAME_ENV: &'static str = "WEATHER_DATA_CONFIG";

    /// Load the default configuration looking at the environment
    pub fn load_default() -> crate::Result<Configuration> {
        // use the default configuration file if it exists otherwise it is a default configuration
        let filename = env_var_or!(Self::FILENAME_ENV, Self::DEFAULT_FILENAME);
        let path = Path::new(&filename);
        if path.exists() && path.is_file() {
            Configuration::try_from(path)
        } else {
            Ok(Configuration::default())
        }
    }
    pub fn save(&self, path: &Path, save_defaults: bool) -> crate::Result<()> {
        let mut document = file::Document::from(self);
        if save_defaults {
            document = document.with_defaults();
        }
        document.save(path)
    }
}

mod file {
    //! The properties file manager.
    use super::*;
    use std::{fs, io::prelude::*};
    use toml;

    /// The structure that holds persisted toml properties document.
    ///
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Document {
        #[serde(rename = "weather-data", default = "weather_data::Document::default")]
        pub weather_data: weather_data::Document,
        #[serde(rename = "visual-crossing", default = "visual_crossing::Document::default")]
        pub visual_crossing: visual_crossing::Document,
        #[serde(rename = "us-cities", default = "us_cities::Document::default")]
        pub us_cities: us_cities::Document,
    }
    impl From<Document> for Configuration {
        /// Create the configuration from the configuration document instance.
        fn from(document: Document) -> Self {
            Configuration {
                weather_data: weather_data::Properties::from(&document.weather_data),
                visual_crossing: visual_crossing::Properties::from(&document.visual_crossing),
                us_cities: us_cities::Properties::from(&document.us_cities),
            }
        }
    }
    impl TryFrom<&Path> for Document {
        type Error = crate::Error;
        fn try_from(path: &Path) -> crate::Result<Self> {
            if !path.exists() {
                err!("'{}' does not exist", path.display())?;
            }
            if !path.is_file() {
                err!("'{}' is not a file", path.display())?;
            }
            match fs::File::open(path) {
                Err(err) => err!("could not open '{}': {}.", path.display(), err),
                Ok(mut file) => {
                    let mut contents = String::new();
                    match file.read_to_string(&mut contents) {
                        Err(err) => err!("could not read '{}' contents: {}.", path.display(), err),
                        Ok(_) => Self::try_from(contents.as_str()),
                    }
                }
            }
        }
    }
    impl TryFrom<&str> for Document {
        type Error = crate::Error;
        fn try_from(properties_str: &str) -> crate::Result<Self> {
            match toml::from_str::<Document>(properties_str) {
                Ok(document) => Ok(document),
                Err(err) => err!("could not load the configuration ({}).", err),
            }
        }
    }
    impl From<&Configuration> for Document {
        fn from(configuration: &Configuration) -> Self {
            Self {
                weather_data: weather_data::Document::from(&configuration.weather_data),
                visual_crossing: visual_crossing::Document::from(&configuration.visual_crossing),
                us_cities: us_cities::Document::from(&configuration.us_cities),
            }
        }
    }
    impl std::fmt::Display for Document {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match toml::to_string(self) {
                Err(error) => write!(f, "{error}"),
                Ok(string) => write!(f, "{string}")
            }
        }
    }
    impl Document {
        /// Create an instance of the document forcing default values as the set property if
        /// the property value is undefined.
        ///
        pub fn with_defaults(&self) -> Document {
            Self {
                weather_data: self.weather_data.with_defaults(),
                visual_crossing: self.visual_crossing.with_defaults(),
                us_cities: self.us_cities.with_defaults(),
            }
        }
        pub fn save(&self, path: &Path) -> crate::Result<()> {
            match toml::to_string(self) {
                Err(error) => err!("could not create configuration contents: {}.", error),
                Ok(contents) => {
                    if let Err(err) = fs::write(path, contents) {
                        err!("could not save the configuration: {}.", err)?;
                    }
                    Ok(())
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::backend::testlib::TestFixture;
        use std::{fs, path::PathBuf};

        #[test]
        fn document() {
            let testcase = Document::default();
            assert!(!testcase.weather_data.directory.get().is_empty());
            assert!(!testcase.visual_crossing.endpoint.get().is_empty());
            assert!(!testcase.visual_crossing.api_key.get().is_empty());
            assert!(!testcase.us_cities.filename.get().is_empty());
        }

        #[test]
        fn toml_document() {
            let set_directory = "set_directory_name";
            // let set_fs_only = !Document::default().weather_data.fs_only();
            let set_fs_only = !Document::default().weather_data.fs_only.get();
            let set_endpoint = "http://set/endpoint";
            let set_api_key = "set_api_key";
            let set_filename = "set_filename.txt";

            let mut document = Document::default();
            document.weather_data.directory.set(set_directory);
            document.weather_data.fs_only.set(set_fs_only);
            document.visual_crossing.endpoint.set(set_endpoint);
            document.visual_crossing.api_key.set(set_api_key);
            document.us_cities.filename.set(set_filename);
            let testcase_str = toml::to_string(&document).unwrap();

            let testcase = Document::try_from(testcase_str.as_str()).unwrap();
            assert_eq!(testcase.weather_data.directory.get(), set_directory);
            assert_eq!(testcase.weather_data.fs_only.get(), set_fs_only);
            assert_eq!(testcase.visual_crossing.endpoint.get(), set_endpoint);
            assert_eq!(testcase.visual_crossing.api_key.get(), set_api_key);
            assert_eq!(testcase.us_cities.filename.get(), set_filename);
        }

        #[test]
        fn properties_from_path() {
            let fixture = TestFixture::create();
            let toml_document = r#"
                [weather-data]
                directory = "toml_directory_name"
                fs-only = true

                [visual-crossing]
                endpoint = "http://toml/endpoint"
                api-key = "toml.api.key"

                [us-cities]
                filename = "toml_filename.csv"
            "#;
            let path = PathBuf::from(&fixture);
            let toml_path = path.join("properties_from_path.toml");
            fs::write(&toml_path, toml_document).unwrap();
            let testcase = Document::try_from(toml_path.as_path()).unwrap();
            assert_eq!(testcase.weather_data.directory.get(), "toml_directory_name");
            assert!(testcase.weather_data.fs_only.get());
            assert_eq!(testcase.visual_crossing.endpoint.get(), "http://toml/endpoint");
            assert_eq!(testcase.visual_crossing.api_key.get(), "toml.api.key");
            assert_eq!(testcase.us_cities.filename.get(), "toml_filename.csv");
        }

        #[test]
        fn save() {
            let mut configuration = Configuration::default();
            configuration.weather_data.directory = "save_directory_name".to_string();
            configuration.weather_data.fs_only = true;
            configuration.visual_crossing.endpoint = "http://save/endpoint".to_string();
            configuration.visual_crossing.api_key = "save.api.key".to_string();
            configuration.us_cities.filename = "save_file_name.csv".to_string();

            let fixture = TestFixture::create();
            let save_filename = PathBuf::from(&fixture).join("save_configuration.toml");
            Document::try_from(&configuration).unwrap().save(&save_filename).unwrap();
            let testcase = Document::try_from(save_filename.as_path()).unwrap();
            assert_eq!(testcase.weather_data.directory.get(), &configuration.weather_data.directory);
            assert_eq!(testcase.weather_data.fs_only.get(), configuration.weather_data.fs_only);
            assert_eq!(testcase.visual_crossing.endpoint.get(), &configuration.visual_crossing.endpoint);
            assert_eq!(testcase.visual_crossing.api_key.get(), &configuration.visual_crossing.api_key);
            assert_eq!(testcase.us_cities.filename.get(), &configuration.us_cities.filename);
        }
    }
}
