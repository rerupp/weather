//! The new version of the weather data API.
use crate::{
    backend::{create, Backend},
    entities::{
        CityFilter, DailyHistories, DateRange, HistoryDates, HistorySummaries, Location, LocationFilter,
        LocationFilters, State,
    },
    histories_future::{self, HistoriesFuture},
    location_filters,
    Result,
};
use std::path::PathBuf;

/// Creates the weather data `API` depending on the backend configuration.
///
/// # Arguments
///
/// * `dirname` is the weather data directory name.
pub fn create_weather_data(config_file: Option<PathBuf>, dirname: Option<PathBuf>, no_db: bool) -> Result<WeatherData> {
    Ok(WeatherData(create(config_file, dirname, no_db)?))
}

/// The weather data `API`.
pub struct WeatherData(
    /// The weather data implementation.
    Box<dyn Backend>,
);
impl WeatherData {
    /// Add weather data history for a location.
    ///
    /// # Arguments
    ///
    /// - `histories` has the location and histories to add.
    ///
    pub fn add_histories(&self, daily_histories: DailyHistories) -> Result<usize> {
        crate::log_elapsed_time!(info, "add_histories");
        self.0.add_daily_histories(daily_histories)
    }

    /// Get weather data history for a location.
    ///
    /// # Arguments
    ///
    /// * `filter` identifies the location.
    /// * `dates` determines which daily histories to get.
    ///
    pub fn new_daily_histories(&self, filter: LocationFilter, dates: DateRange) -> Result<HistoriesFuture> {
        // get the locations existing history dates
        let mut history_dates = self.0.get_history_dates(location_filters!(filter))?;
        if history_dates.len() > 1 {
            Err("More than 1 location was found.")?;
        }
        let location_history_dates = history_dates.pop().unwrap();
        histories_future::get(dates, location_history_dates, self.0.get_config())
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
    pub fn get_daily_histories(&self, filter: LocationFilter, history_range: DateRange) -> Result<DailyHistories> {
        crate::log_elapsed_time!(info, "get_daily_history");
        self.0.get_daily_histories(location_filters![filter], history_range)
    }

    /// Get the history dates for locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations.
    ///
    pub fn get_history_dates(&self, filters: LocationFilters) -> Result<Vec<HistoryDates>> {
        crate::log_elapsed_time!(info, "get_history_dates");
        self.0.get_history_dates(filters)
    }

    /// Get a summary of location weather data.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations.
    ///
    pub fn get_history_summary(&self, filters: LocationFilters) -> Result<Vec<HistorySummaries>> {
        crate::log_elapsed_time!(info, "get_history_summary");
        self.0.get_history_summaries(filters)
    }

    /// Get the weather location metadata.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations of interest.
    ///
    pub fn get_locations(&self, filters: LocationFilters) -> Result<Vec<Location>> {
        crate::log_elapsed_time!(info, "get_locations");
        self.0.get_locations(filters)
    }

    /// Add a location to weather data.
    ///
    /// # Arguments
    ///
    /// - `location` is the location that will be added.
    ///
    pub fn add_location(&self, location: Location) -> Result<()> {
        crate::log_elapsed_time!(info, "add_location");
        self.0.add_location(location)
    }

    /// Search for locations that can be added to weather data.
    ///
    /// # Arguments
    ///
    /// - `criteria` provides the search parameters.
    ///
    pub fn search_locations(&self, filter: CityFilter) -> Result<Vec<Location>> {
        crate::log_elapsed_time!(info, "search_locations");
        self.0.search_locations(filter)
    }

    /// Get the state metadata for US Cities.
    ///
    pub fn get_states(&self) -> Result<Vec<State>> {
        crate::log_elapsed_time!(info, "get_states");
        self.0.get_states()
    }
}
