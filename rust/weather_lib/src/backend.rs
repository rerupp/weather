//! The weather data user API implementations.
//! 
//! The [Backend] trait defines the API [weather_data](crate::weather_data) uses to 
//! access and update weather history.

pub(crate) mod admin;
mod db;
mod filesys;
pub(crate) use filesys::WeatherDir;

use crate::prelude::{
    City, Configuration, DailyHistories, DateRange, HistoryDates, HistorySummary, Location, LocationFilter, State,
};
use std::sync::Arc;

/// Get the backend implementation of weather data.
///
/// # Arguments
///
/// * `configuration` is the weather data configuration properties.
///
pub fn create_backend(configuration: Arc<Configuration>) -> crate::Result<Box<dyn Backend>> {
    if configuration.weather_data.fs_only {
        filesys::create_filesys_backend(configuration)
    } else {
        let weather_dir = WeatherDir::try_from(&configuration)?;
        if db::is_available(&weather_dir) {
            db::create_db_backend(configuration)
        } else {
            filesys::create_filesys_backend(configuration)
        }
    }
}

/// The weather data API for backend implementations.
///
pub(crate) trait Backend: Send + Sync {
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
    /// - `filter` identifies the location.
    /// - `history_range` covers the history dates returned.
    ///
    fn get_daily_histories(&self, filter: LocationFilter, history_range: DateRange) -> crate::Result<DailyHistories>;

    /// Get the history dates for locations.
    ///
    /// # Arguments
    ///
    /// - `filters_opt` identifies the locations.
    ///
    fn get_history_dates(&self, filters_opt: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistoryDates>>;

    /// Get a summary of location weather data.
    ///
    /// # Arguments
    ///
    /// - `filters_opt` identifies the locations.
    ///
    fn get_history_summaries(&self, filters_opt: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistorySummary>>;

    /// Get the weather location metadata.
    ///
    /// # Arguments
    ///
    /// - `filters_opt` identifies the locations of interest.
    ///
    fn get_locations(&self, filters_opt: Option<Vec<LocationFilter>>) -> crate::Result<Vec<Location>>;

    /// Add a location.
    ///
    /// #Arguments
    ///
    /// * `location` is the location data.
    ///
    fn add_location(&self, location: Location) -> crate::Result<()>;

    /// Delete a location.
    ///
    /// # Arguments
    ///
    /// * `filter` is used to get the location.
    ///
    fn delete_location(&self, filter: LocationFilter) -> crate::Result<()>;

    /// Update a location properties.
    ///
    /// # Arguments
    ///
    /// * `location` identifies the location and contains the new property values.
    ///
    fn update_location(&self, location: Location) -> crate::Result<bool>;

    /// Search the Cities database for locations.
    ///
    /// # Arguments
    ///
    /// * `filters_opt` is used to find cities.
    /// * `limit` restricts the number of cities returned.
    ///
    #[allow(unused_variables)]
    fn get_cities(&self, filters_opt: Option<Vec<LocationFilter>>, limit: usize) -> crate::Result<Vec<City>> {
        Err(crate::Error("Get cities is not available.".to_string()))
    }

    /// Get a list of the US City states.
    ///
    fn get_states(&self) -> crate::Result<Vec<State>> {
        Err(crate::Error("Get states is not available.".to_string()))
    }
}

#[cfg(test)]
pub(crate) mod testlib {
    //! A library for common utilities used by the backend.

    use crate::backend::WeatherDir;
    use rand::Rng;
    use std::{env, fmt, fs, path};

    /// Used to create a temporary weather directory and delete it as part of the function exit.
    #[derive(Debug)]
    pub(crate) struct TestFixture(path::PathBuf);
    impl TestFixture {
        /// Creates a test weather directory or panics if a unique directory cannot be created.
        pub(crate) fn create() -> Self {
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
        pub(crate) fn copy_resources(&self, source: &path::PathBuf) {
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
    impl From<&TestFixture> for WeatherDir {
        fn from(fixture: &TestFixture) -> Self {
            WeatherDir::try_from(fixture.to_string()).unwrap()
        }
    }

    pub(crate) fn generate_random_string(len: usize) -> String {
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

    pub(crate) fn test_resources() -> path::PathBuf {
        path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources").join("tests")
    }
}
