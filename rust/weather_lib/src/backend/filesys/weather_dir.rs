//! This module provides a helper for getting weather data files from a directory in a filesystem.

use crate::{
    backend::{filesys::WeatherFile, Configuration},
    Error, Result,
};
use std::path::{Path, PathBuf};

#[doc(hidden)]
macro_rules! err {
    ($id:expr, $reason:expr) => {
        Error::from(format!("WeatherDir ({}): {}", $id, $reason))
    };
}

/// The manager responsible for stat, readers, and writers to file contents in the weather directory
#[derive(Clone, Debug)]
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

impl TryFrom<&Configuration> for WeatherDir {
    type Error = Error;
    fn try_from(configuration: &Configuration) -> std::result::Result<Self, Self::Error> {
        WeatherDir::new(PathBuf::from(&configuration.weather_data.directory))
    }
}
impl TryFrom<&std::sync::Arc<Configuration>> for WeatherDir {
    type Error = Error;
    fn try_from(configuration: &std::sync::Arc<Configuration>) -> std::result::Result<Self, Self::Error> {
        WeatherDir::new(PathBuf::from(&configuration.weather_data.directory))
    }
}

/// This is frontend to a filesystem directory containing weather data.
///
impl WeatherDir {
    pub const ARCHIVE_EXTENSION: &'static str = "zip";

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
            false => Err(err!(path.display().to_string(), "Not a directory...")),
        }
    }
    /// Get a weather file from within the managed directory.
    ///
    /// # Arguments
    ///
    /// * `filename` is the name of the file within the weather directory.
    ///
    pub fn file(&self, filename: &str) -> WeatherFile {
        WeatherFile::new(self.0.join(filename))
    }

    /// Get an archive from the weather data directory.
    ///
    /// # Arguments
    ///
    /// * `alias` is the location alias name.
    ///
    pub fn archive(&self, alias: &str) -> WeatherFile {
        let archive_name = self.0.join(alias).with_extension(Self::ARCHIVE_EXTENSION);
        WeatherFile::new(archive_name)
    }

    /// The admin api needs access to the path.
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
        assert_eq!(file.size(), 899);
    }
}
