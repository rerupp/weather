//! The `Python` class that front ends weather data.

use super::*;

use py_entities::*;
use py_history_client::PyHistoryClient;
use std::sync::OnceLock;
use toolslib::{fmt::commafy, logs, stopwatch::StopWatch};
use weather_lib::prelude::WeatherData;

pub struct ElapsedTimer {
    banner: String,
    stopwatch: StopWatch,
}
impl Drop for ElapsedTimer {
    fn drop(&mut self) {
        let ms = self.stopwatch.elapsed().as_millis();
        match ms == 0 {
            true => log::debug!("{} exit duration {}us", self.banner, self.stopwatch.elapsed().as_micros()),
            false => log::debug!("{} exit duration {}ms", self.banner, commafy(self.stopwatch.elapsed().as_millis())),
        }
    }
}
impl ElapsedTimer {
    pub fn new(banner: impl ToString) -> Self {
        let banner = banner.to_string();
        log::debug!("{} enter", banner);
        Self { banner, stopwatch: StopWatch::start_new() }
    }
}
macro_rules! elapsed_timer {
    ($banner:expr) => {
        let __elapsed_timer__ = ElapsedTimer::new($banner);
    };
}

/// Creates the weather data `API` depending on the backend configuration.
///
/// # Arguments
///
/// * `init` is the weather data initializer.
///
#[pyfunction]
// #[pyo3(text_signature = "(init, /)")]
pub fn create(init: PyWeatherConfig) -> PyResult<PyWeatherData> {
    // track if logging has already been initialized or not (this is not thread safe)
    static LOG_INITIALIZED: OnceLock<bool> = OnceLock::new();
    if LOG_INITIALIZED.get().is_none() {
        let log_properties = logs::LogProperties {
            level: match init.log_level {
                0 => log::LevelFilter::Warn,
                1 => log::LevelFilter::Info,
                2 => log::LevelFilter::Debug,
                _ => log::LevelFilter::Trace,
            },
            console_pattern: None,
            logfile_pattern: None,
            logfile_path: init.logfile,
            logfile_append: init.log_append,
            file_loggers: vec!["toolslib".to_string(), "weather_lib".to_string(), "py_weather_lib".to_string()],
        };
        match logs::initialize(log_properties) {
            Ok(_) => LOG_INITIALIZED.set(true).unwrap(),
            Err(error) => system_err!(error)?,
        }
    }
    match weather_lib::create_weather_data(init.config_file, init.dirname, init.fs_only) {
        Ok(weather_data) => {
            log::debug!("created weather data");
            Ok(PyWeatherData(weather_data))
        }
        Err(error) => system_err!(error),
    }
}

/// The weather data `API`.
#[pyclass]
pub struct PyWeatherData(WeatherData);
#[pymethods]
impl PyWeatherData {
    /// Add weather data history for a location.
    ///
    /// # Arguments
    ///
    /// - `histories` has the location and histories to add.
    ///
    pub fn add_histories(&self, daily_histories: PyDailyHistories) -> PyResult<usize> {
        match self.0.add_histories(daily_histories.into()) {
            Ok(count) => Ok(count),
            Err(error) => system_err!(error),
        }
    }
    /// Get the client that retrieves weather history for a location.
    ///
    pub fn get_history_client(&self) -> PyResult<PyHistoryClient> {
        match self.0.get_history_client() {
            Ok(history_client) => Ok(PyHistoryClient::new(history_client)),
            Err(error) => system_err!(error),
        }
    }
    /// Get daily weather history for a location.
    ///
    /// It is an error if more than 1 location is found.
    ///
    /// # Arguments
    ///
    /// * `filter` identifies the location.
    /// * `history_range` covers the history dates returned.
    ///
    pub fn get_daily_history(
        &self,
        filter: PyLocationFilter,
        history_range: PyDateRange,
    ) -> PyResult<PyDailyHistories> {
        elapsed_timer!("get_daily_history");
        match self.0.get_daily_history(filter.into(), history_range.into()) {
            Ok(daily_histories) => Ok(daily_histories.into()),
            Err(error) => system_err!(error),
        }
    }
    /// Get the history dates for locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations.
    ///
    pub fn get_history_dates(&self, filters: PyLocationFilters) -> PyResult<Vec<PyHistoryDates>> {
        elapsed_timer!("get_history_dates");
        match self.0.get_history_dates(filters.into()) {
            Ok(history_dates) => Ok(history_dates.into_iter().map(Into::into).collect()),
            Err(error) => system_err!(error),
        }
    }
    /// Get a summary of location weather data.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations.
    ///
    pub fn get_history_summary(&self, filters: PyLocationFilters) -> PyResult<Vec<PyHistorySummaries>> {
        elapsed_timer!("get_history_summary");
        match self.0.get_history_summary(filters.into()) {
            Ok(history_summary) => Ok(history_summary.into_iter().map(Into::into).collect()),
            Err(error) => system_err!(error),
        }
    }
    /// Get the weather location metadata.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations of interest.
    ///
    pub fn get_locations(&self, filters: PyLocationFilters) -> PyResult<Vec<PyLocation>> {
        elapsed_timer!("get_locations");
        match self.0.get_locations(filters.into()) {
            Ok(locations) => Ok(locations.into_iter().map(Into::into).collect()),
            Err(error) => system_err!(error),
        }
    }
    /// Add a location to weather data.
    ///
    /// # Arguments
    ///
    /// - `location` is the location that will be added.
    ///
    pub fn add_location(&self, location: PyLocation) -> PyResult<()> {
        elapsed_timer!("add_location");
        match self.0.add_location(location.into()) {
            Ok(_) => Ok(()),
            Err(error) => system_err!(error),
        }
    }

    /// Search for locations that can be added to weather data.
    ///
    /// # Arguments
    ///
    /// - `filter` is used to select cities and limit how many are returned as a location..
    ///
    pub fn search_locations(&self, filter: PyCityFilter) -> PyResult<Vec<PyLocation>> {
        elapsed_timer!("search_locations");
        match self.0.search_locations(filter.into()) {
            Ok(locations) => Ok(locations.into_iter().map(Into::into).collect()),
            Err(error) => system_err!(error),
        }
    }

    /// Get the state metadata for US Cities.
    ///
    pub fn get_states(&self) -> PyResult<Vec<PyState>> {
        elapsed_timer!("get_states");
        match self.0.get_states() {
            Ok(states) => Ok(states.into_iter().map(Into::into).collect()),
            Err(error) => system_err!(error),
        }
    }
}
