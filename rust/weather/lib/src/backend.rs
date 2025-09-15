//! The implementations of weather data.

mod db;
mod filesys;

pub use config::Config;
pub mod admin;
mod config;

use crate::prelude::{
    DailyHistories, DateRange, HistoryDates, HistorySummaries, Location, LocationFilters, State, CityFilter,
};
use std::path::PathBuf;

/// Get the backend implementation of weather data.
///
/// # Arguments
///
/// * `config_file` is the weather data configuration filename.
/// * `dirname` is the weather data directory name override.
/// * `no_db` forces the filesys backend to be used.
///
pub fn create(config_file: Option<PathBuf>, dirname: Option<PathBuf>, no_db: bool) -> crate::Result<Box<dyn Backend>> {
    let mut config = Config::new(config_file)?;
    if let Some(path) = dirname {
        config.weather_data.directory = path.display().to_string();
    }
    let weather_dir = filesys::WeatherDir::try_from(&config)?;
    if no_db {
        filesys::create_filesys_backend(config)
    } else if db::is_available(&weather_dir) {
        db::create_db_backend(config)
    } else {
        filesys::create_filesys_backend(config)
    }
}

/// The weather data API for backend implementations.
///
pub(crate) trait Backend: Send {
    /// Get the weather data configuration.
    ///
    fn get_config(&self) -> &Config;

    /// Add weather data history to a location.
    ///
    /// # Arguments
    ///
    /// - `daily_histories` contains the historical weather data that will be added.
    ///
    // todo: should this return the dates added instead?
    fn add_daily_histories(&self, daily_histories: DailyHistories) -> crate::Result<usize>;

    /// Get daily weather history for a location.
    ///
    /// It is an error if more than 1 location is found.
    ///
    /// # Arguments
    ///
    /// - `filters` identifies the location.
    /// - `history_range` covers the history dates returned.
    ///
    // todo: change this to allow multiple locations or change to the location alias
    fn get_daily_histories(&self, filters: LocationFilters, history_range: DateRange) -> crate::Result<DailyHistories>;

    /// Get the history dates for locations.
    ///
    /// # Arguments
    ///
    /// - `filters` identifies the locations.
    ///
    fn get_history_dates(&self, filters: LocationFilters) -> crate::Result<Vec<HistoryDates>>;

    /// Get a summary of location weather data.
    ///
    /// # Arguments
    ///
    /// - `filters` identifies the locations.
    ///
    fn get_history_summaries(&self, filters: LocationFilters) -> crate::Result<Vec<HistorySummaries>>;

    /// Get the weather location metadata.
    ///
    /// # Arguments
    ///
    /// - `filters` identifies the locations of interest.
    ///
    fn get_locations(&self, filters: LocationFilters) -> crate::Result<Vec<Location>>;

    /// Add a location.
    ///
    /// #Arguments
    ///
    /// * `location` is the location data.
    ///
    fn add_location(&self, location: Location) -> crate::Result<()>;

    /// Search for a location.
    ///
    /// # Arguments
    ///
    /// * `filter` identifies which cities are being searched for (default is all).
    ///
    fn search_locations(&self, filter: CityFilter) -> crate::Result<Vec<Location>>;

    /// Get a list of the US City states.
    ///
    fn get_states(&self) -> crate::Result<Vec<State>>;
}

#[cfg(test)]
mod testlib {
    //! A library for common utilities used by the backend.

    use rand::Rng;
    use std::{env, fmt, fs, path};

    /// Used to create a temporary weather directory and delete it as part of the function exit.
    #[derive(Debug)]
    pub(in crate::backend) struct TestFixture(path::PathBuf);
    impl TestFixture {
        /// Creates a test weather directory or panics if a unique directory cannot be created.
        pub(in crate::backend) fn create() -> Self {
            let tmpdir = env::temp_dir();
            let mut weather_dir: Option<path::PathBuf> = None;
            // try to create a test directory 10 times
            for _ in [0..10] {
                let test_dir = tmpdir.join(format!("weather_dir-{}", generate_random_string(15)));
                match test_dir.exists() {
                    true => {
                        eprintln!("Test directory '{}' exists...", test_dir.as_path().display())
                    }
                    false => {
                        weather_dir.replace(test_dir);
                        break;
                    }
                }
            }
            match weather_dir {
                Some(root_dir) => match fs::create_dir(&root_dir) {
                    Ok(_) => Self(root_dir),
                    Err(e) => {
                        panic!("Error creating '{}': {}", root_dir.as_path().display(), e.to_string())
                    }
                },
                None => panic!("Tried 10 times to get a unique test directory name and failed..."),
            }
        }
        pub(in crate::backend) fn copy_resources(&self, source: &path::PathBuf) {
            if source.is_file() {
                let target = self.0.join(source.file_name().unwrap().to_str().unwrap());
                if let Err(err) = fs::copy(source, &target) {
                    panic!("Error copying {} to {} ({}).", source.as_path().display(), self, &err);
                }
            } else {
                let paths = fs::read_dir(&source).unwrap();
                for entry in paths {
                    let source_path = entry.unwrap().path();
                    let target_path = self.0.join(source_path.file_name().unwrap().to_str().unwrap());
                    if let Err(err) = fs::copy(&source_path, &target_path) {
                        panic!("Error copying {} to {} ({}).", source_path.as_path().display(), self, &err);
                    }
                }
            }
        }
    }
    impl Drop for TestFixture {
        /// Clean up the temporary directory as best you can.
        fn drop(&mut self) {
            if let Err(e) = fs::remove_dir_all(self.to_string()) {
                eprintln!("Yikes... Error cleaning up test weather_dir: {}", e.to_string());
            }
        }
    }
    impl fmt::Display for TestFixture {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0.as_path().display())
        }
    }
    impl From<&TestFixture> for path::PathBuf {
        fn from(value: &TestFixture) -> Self {
            path::PathBuf::from(value.to_string())
        }
    }

    pub(in crate::backend) fn generate_random_string(len: usize) -> String {
        let mut rand = rand::rng();
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmonopqrstuvwxyz0123456789";
        let random_string = (0..len)
            .map(|_| {
                let idx = rand.random_range(0..CHARS.len());
                CHARS[idx] as char
            })
            .collect();
        // eprintln!("generate_random_string: {}...", random_string);
        random_string
    }

    pub(in crate::backend) fn test_resources() -> path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources").join("tests")
    }
}
