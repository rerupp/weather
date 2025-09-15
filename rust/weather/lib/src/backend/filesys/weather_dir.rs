#![allow(unused)]

use crate::{
    backend::{filesys::WeatherFile, Config},
    Error, Result,
};
use std::path::{Path, PathBuf};

/// The [crate::backend::filesys::WeatherDir] error builder.
macro_rules! error {
    ($id:expr, $reason:expr) => {
        Error::from(format!("WeatherDir ({}): {}", $id, $reason))
    };
}

/// The manager responsible for stat, readers, and writers to file contents in the weather directory
#[derive(Debug)]
pub struct WeatherDir(
    /// The directory managed by the weather directory.
    PathBuf,
);

impl std::fmt::Display for WeatherDir {
    /// Use this trait to expose the weather directory pathname.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}", self.0.as_path().display())
        write!(f, "{}", self.0.as_path().display())
    }
}

impl TryFrom<String> for WeatherDir {
    type Error = Error;
    /// Create a [WeatherDir] instance using the string as a directory pathname.
    fn try_from(dirname: String) -> std::result::Result<Self, Self::Error> {
        WeatherDir::new(PathBuf::from(dirname))
    }
}

impl TryFrom<&str> for WeatherDir {
    type Error = Error;
    /// Create a [WeatherDir] instance using the string as a directory pathname.
    fn try_from(dirname: &str) -> std::result::Result<Self, Self::Error> {
        WeatherDir::new(PathBuf::from(dirname))
    }
}

impl TryFrom<&Config> for WeatherDir {
    type Error = Error;
    fn try_from(config: &Config) -> std::result::Result<Self, Self::Error> {
        WeatherDir::new(PathBuf::from(&config.weather_data.directory))
    }
}

impl WeatherDir {
    /// Creates a new instance of the weather directory manager.
    ///
    /// An error will be returned if the directory does not exist, or does exist but is not a directory.
    ///
    /// # Arguments
    ///
    /// * `directory_name` is the name of the directory.
    pub fn new(path: PathBuf) -> Result<WeatherDir> {
        match path.is_dir() {
            true => Ok(WeatherDir(path)),
            false => Err(error!(path.display().to_string(), "Not a directory...")),
        }
    }
    /// Get a weather file from within the managed directory.
    ///
    /// # Arguments
    ///
    /// * `filename` is the name of the file within the weather directory.
    pub fn file(&self, filename: &str) -> WeatherFile {
        WeatherFile::new(self.0.join(filename))
    }
    pub fn archive(&self, alias: &str) -> WeatherFile {
        let archive_name = self.0.join(alias).with_extension("zip");
        WeatherFile::new(archive_name)
    }
    /// Get the weather directory path.
    pub fn path(&self) -> &Path {
        self.0.as_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testlib;

    #[test]
    fn weather_dir() {
        // set up the test case
        let fixture = testlib::TestFixture::create();
        let filename = "locations.json";
        let resource = testlib::test_resources().join("filesys").join(filename);
        fixture.copy_resources(&resource);
        // now spot check it
        let testcase = WeatherDir::try_from(fixture.to_string()).unwrap();
        let file = testcase.file(filename);
        assert!(file.exists());
        assert_eq!(file.size(), 664);
    }
}
