//! The weather data configuration table.

use super::*;

/// The weather data properties.
///
#[derive(Clone, Debug)]
pub struct Properties {
    /// The weather data directory name.
    pub directory: String,
    /// When `true` only use the filesystem data regardless if a database is available.
    pub fs_only: bool,
    /// The maximum number of workers to use when running bulk operations.
    pub max_workers: usize,
}
impl Default for Properties {
    fn default() -> Self {
        Self::from(&Document::default())
    }
}
impl From<&Document> for Properties {
    fn from(document: &Document) -> Self {
        Self {
            directory: document.directory.get().to_string(),
            fs_only: document.fs_only.get(),
            max_workers: document.max_workers.get(),
        }
    }
}

macros::string_property!(Directory, "directory", Document::default_directory(), "Document::default_directory");
macros::bool_property!(FsOnly, "fs-only", Document::default_fs_only(), "Document::default_fs_only");
macros::usize_property!(MaxWorkers, "max-workers", Document::default_max_workers(), "Document::default_max_workers");

/// The weather data configuration that can be serialized and deserialized.
///
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Document {
    /// The weather data directory name.
    #[serde(flatten)]
    pub directory: Directory,

    /// When `true` only use the filesystem data regardless if a database is available.
    #[serde(flatten)]
    pub fs_only: FsOnly,

    /// Determines the maximum number of threads to use when running bulk operations.
    #[serde(flatten)]
    pub max_workers: MaxWorkers,
}
impl From<&Properties> for Document {
    fn from(properties: &Properties) -> Self {
        let mut self_ = Document::default();
        self_.directory.set(properties.directory.as_str());
        self_.fs_only.set(properties.fs_only);
        self_.max_workers.set(properties.max_workers);
        self_
    }
}
impl Document {
    /// Initialize unset property values to their default value.
    ///
    pub fn with_defaults(&self) -> Self {
        Self {
            directory: self.directory.with_default(),
            fs_only: self.fs_only.with_default(),
            max_workers: self.max_workers.with_default(),
        }
    }

    /// The environment variable that contains the default weather data directory name.
    const ENV_DIRECTORY: &'static str = "WEATHER_DATA";
    /// The default weather data directory name.
    const DEFAULT_DIRECTORY: &'static str = "weather_data";
    /// Gets the weather data directory from the process environment if [Self::ENV_DIRECTORY] is defined
    /// otherwise [Self::DEFAULT_DIRECTORY].
    fn default_directory() -> String {
        env_var_or!(Self::ENV_DIRECTORY, Self::DEFAULT_DIRECTORY)
    }

    /// The environment variable that holds the default filesystem only flag.
    const ENV_FS_ONLY: &'static str = "WEATHER_DATA_FS_ONLY";
    /// The default filesystem only flag.
    const DEFAULT_FS_ONLY: bool = false;
    /// Gets the filesystem only flag from the process environment if [Self::ENV_FS_ONLY] is defined
    /// otherwise [Self::DEFAULT_FS_ONLY].
    fn default_fs_only() -> bool {
        str_to_bool(&env_var_or!(Self::ENV_FS_ONLY, Self::DEFAULT_FS_ONLY))
    }

    /// The environment variable that holds the default filesystem only flag.
    const ENV_MAX_WORKERS: &'static str = "WEATHER_DATA_MAX_WORKERS";
    /// The default filesystem only flag.
    const DEFAULT_MAX_WORKERS: usize = 16;
    /// Gets the filesystem only flag from the process environment if [Self::ENV_MAX_WORKERS] is defined
    /// otherwise [Self::DEFAULT_MAX_WORKERS].
    fn default_max_workers() -> usize {
        let max_workers_str = env_var_or!(Self::ENV_MAX_WORKERS, Self::DEFAULT_MAX_WORKERS);
        let max_workers = max_workers_str.parse::<usize>().unwrap_or_else(|_| {
            log::error!("Could not parse {} ({max_workers_str}) as usize, using default.", Self::ENV_MAX_WORKERS);
            Self::DEFAULT_MAX_WORKERS
        });
        // cap the env to be a reasonable number of workers otherwise require a configuration file
        std::cmp::max(max_workers, 32)
    }
}

/// Used internally to convert a string into true or false.
///
fn str_to_bool(s: &str) -> bool {
    let lc = s.trim().to_lowercase();
    match lc.is_empty() {
        true => false,
        false => match lc.as_str() {
            "true" | "yes" | "ok" => true,
            num => match num.parse::<i64>() {
                Ok(i) => i != 0,
                _ => false,
            },
        },
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn bool_from_str() {
        assert!(str_to_bool("TRUE"));
        assert!(str_to_bool("true"));
        assert!(str_to_bool("TruE"));
        assert!(str_to_bool("YES"));
        assert!(str_to_bool("yes"));
        assert!(str_to_bool("YeS"));
        assert!(str_to_bool("OK "));
        assert!(str_to_bool(" ok"));
        assert!(str_to_bool(" Ok "));
        assert!(!str_to_bool(String::default().as_str()));
        assert!(!str_to_bool(" "));
        assert!(!str_to_bool("any-string"));
        assert!(str_to_bool("1"));
        assert!(!str_to_bool("0"));
    }

    #[test]
    fn env_document() {
        // todo: uncomment this test once config is removed
        // // this should be the only test where the process environment needs to be threadsafe
        // let env_directory = "env_directory";
        // let env_fs_only = !default_fs_only();
        // env::set_var(ENV_DIRECTORY, env_directory);
        // env::set_var(ENV_FS_ONLY, env_fs_only.to_string());
        // let testcase = Document::default();
        // env::remove_var(ENV_DIRECTORY);
        // env::remove_var(ENV_FS_ONLY);
        //
        // // these tests are fragile due to the process environment
        // assert_eq!(testcase.default_directory, env_directory);
        // assert_eq!(testcase.default_fs_only, env_fs_only);
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    struct DocumentTestcase {
        #[serde(rename = "weather-data", default = "Document::default")]
        weather_data: Document,
    }

    #[test]
    fn toml_document() {
        // these tests are fragile because the process environment is used

        // no document
        let default = Document::default();
        let testcase = toml::from_str::<DocumentTestcase>("").unwrap();
        assert_eq!(default.directory.get(), testcase.weather_data.directory.get());
        assert_eq!(default.fs_only.get(), testcase.weather_data.fs_only.get());

        // default document
        let default = DocumentTestcase::default();
        let testcase_str = toml::to_string(&default).unwrap();
        let testcase = toml::from_str::<DocumentTestcase>(&testcase_str).unwrap();
        assert_eq!(default.weather_data.directory.get(), testcase.weather_data.directory.get());
        assert_eq!(default.weather_data.fs_only.get(), testcase.weather_data.fs_only.get());

        // non-default values
        let set_directory = "toml_directory";
        let set_fs_only = !Document::default_fs_only();
        let mut testcase = Document::default();
        testcase.directory.set(set_directory);
        testcase.fs_only.set(set_fs_only);
        let testcase_str = toml::to_string(&DocumentTestcase { weather_data: testcase }).unwrap();

        let document_testcase = toml::from_str::<DocumentTestcase>(&testcase_str).unwrap();
        assert_eq!(document_testcase.weather_data.directory.get(), set_directory);
        assert_eq!(document_testcase.weather_data.fs_only.get(), set_fs_only);
    }

    #[test]
    fn properties() {
        let mut document = Document::default();
        assert_eq!(document.directory.get(), Document::default_directory());
        assert_eq!(document.fs_only.get(), Document::default_fs_only());
        assert_eq!(document.max_workers.get(), Document::default_max_workers());

        let set_directory = "set_directory";
        let set_fs_only = !Document::default_fs_only();
        let set_max_workers = 128;
        document.directory.set(set_directory);
        document.fs_only.set(set_fs_only);
        document.max_workers.set(set_max_workers);
        let testcase = Properties::from(&document);
        assert_eq!(testcase.directory, set_directory);
        assert_eq!(testcase.fs_only, set_fs_only);
        assert_eq!(testcase.max_workers, set_max_workers);

        // these tests are fragile because the process environment is not threadsafe
        let test_document = Document::from(&testcase);
        assert_eq!(test_document.directory.get(), set_directory);
        assert_eq!(test_document.fs_only.get(), set_fs_only);
        assert_eq!(test_document.max_workers.get(), set_max_workers);
    }

    #[test]
    fn with_defaults() {
        // make sure the defaults show up
        let document = Document::default();
        assert!(document.directory.value.is_none());
        assert!(document.fs_only.value.is_none());
        assert!(document.max_workers.value.is_none());
        let testcase = document.with_defaults();
        assert_eq!(testcase.directory.value.unwrap(), document.directory.default_value);
        assert_eq!(testcase.fs_only.value.unwrap(), document.fs_only.default_value);
        assert_eq!(testcase.max_workers.value.unwrap(), document.max_workers.default_value);

        // make sure you don't clobber properties already set
        let mut document = Document::default();
        let directory = "testcase_dir";
        let fs_only = !Document::default_fs_only();
        let max_workers = 24;
        document.directory.set(directory);
        document.fs_only.set(fs_only);
        document.max_workers.set(max_workers);
        assert_eq!(document.directory.value.as_ref().unwrap(), directory);
        assert_eq!(document.fs_only.value.as_ref().unwrap(), &fs_only);
        assert_eq!(*document.max_workers.value.as_ref().unwrap(), max_workers);
        let testcase = document.with_defaults();
        assert_eq!(testcase.directory.value.as_ref().unwrap(), directory);
        assert_eq!(testcase.fs_only.value.as_ref().unwrap(), &fs_only);
        assert_eq!(*testcase.max_workers.value.as_ref().unwrap(), max_workers);
    }
}
