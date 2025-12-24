//! The new version of the weather data API.
use crate::{
    backend::{create_backend, Backend},
    histories_future,
    prelude::{
        CityFilter, Configuration, DailyHistories, DateRange, HistoriesFuture, HistoryDates, HistorySummaries,
        Location, LocationFilter, State,
    },
};
use std::{path::PathBuf, sync::Arc};

/// Creates the weather data `API` depending on the backend configuration.
///
/// # Arguments
///
/// * `dirname` is the weather data directory name.
pub fn create_weather_data(
    configuration_file: Option<PathBuf>,
    directory_name: Option<PathBuf>,
    fs_only: bool,
) -> crate::Result<WeatherData> {
    let mut configuration = if let Some(file) = configuration_file {
        Configuration::try_from(file.as_path())?
    } else {
        Configuration::load_default()?
    };
    if let Some(directory) = directory_name {
        configuration.weather_data.directory = directory.display().to_string();
    }
    configuration.weather_data.fs_only = fs_only;
    WeatherData::try_from(configuration)
}

/// The weather history data `API`.
///
pub struct WeatherData {
    /// The weather data configuration.
    configuration: Arc<Configuration>,
    /// The weather data implementation.
    pub(crate) backend: Box<dyn Backend>,
}
impl TryFrom<Configuration> for WeatherData {
    type Error = crate::Error;
    fn try_from(configuration: Configuration) -> Result<Self, Self::Error> {
        // the configuration needs to be thread safe because the py_lib API is thread safe
        let configuration = Arc::new(configuration);
        let backend = create_backend(configuration.clone())?;
        Ok(Self { configuration, backend })
    }
}
impl WeatherData {
    /// Add weather data history for a location.
    ///
    /// # Arguments
    ///
    /// - `histories` has the location and histories to add.
    ///
    pub fn add_histories(&self, daily_histories: DailyHistories) -> crate::Result<usize> {
        crate::log_elapsed_time!(info, "add_histories");
        self.backend.add_daily_histories(daily_histories)
    }

    /// Get new weather data history for a location.
    ///
    /// # Arguments
    ///
    /// * `filter` establishes the location.
    /// * `dates` provides the start and end date for the new weather data history.
    ///
    // todo: rename this to new_daily_histories
    pub fn fetch_daily_histories(&self, filter: LocationFilter, dates: DateRange) -> crate::Result<HistoriesFuture> {
        // get the locations existing history dates
        let mut history_dates = self.backend.get_history_dates(Some(vec![filter]))?;
        if history_dates.len() > 1 {
            Err("More than 1 location was found.")?;
        }
        let location_history_dates = history_dates.pop().unwrap();
        histories_future::get(dates, location_history_dates, &self.configuration)
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
    pub fn get_daily_histories(
        &self,
        filter: LocationFilter,
        history_range: DateRange,
    ) -> crate::Result<DailyHistories> {
        crate::log_elapsed_time!(info, "get_daily_history");
        self.backend.get_daily_histories(filter, history_range)
    }

    /// Get the history dates for locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations.
    ///
    pub fn get_history_dates(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistoryDates>> {
        crate::log_elapsed_time!(info, "get_history_dates");
        self.backend.get_history_dates(filters)
    }

    /// Get a summary of location weather data.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations.
    ///
    pub fn get_history_summary(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistorySummaries>> {
        crate::log_elapsed_time!(info, "get_history_summary");
        self.backend.get_history_summaries(filters)
    }

    /// Get the weather location metadata.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations of interest.
    ///
    pub fn get_locations(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<Location>> {
        crate::log_elapsed_time!(info, "get_locations");
        self.backend.get_locations(filters)
    }

    /// Add a location to weather data.
    ///
    /// # Arguments
    ///
    /// - `location` is the location that will be added.
    ///
    pub fn add_location(&self, location: Location) -> crate::Result<()> {
        crate::log_elapsed_time!(info, "add_location");
        self.backend.add_location(location)
    }

    /// Update a locations properties.
    ///
    /// # Arguments
    ///
    /// * `location` contains the locations new property values.
    ///
    pub fn update_location(&self, location: Location) -> crate::Result<bool> {
        self.backend.update_location(location)
    }

    /// Delete a location from weather history.
    ///
    /// # Arguments
    ///
    /// * `location` contains the locations new property values.
    ///
    pub fn delete_location(&self, filter: LocationFilter) -> crate::Result<()> {
        self.backend.delete_location(filter)
    }

    /// Search for locations that can be added to weather data.
    ///
    /// # Arguments
    ///
    /// - `criteria` provides the search parameters.
    ///
    pub fn search_locations(&self, filter: CityFilter) -> crate::Result<Vec<Location>> {
        crate::log_elapsed_time!(info, "search_locations");
        self.backend.search_locations(filter)
    }

    /// Get the state metadata for US Cities.
    ///
    pub fn get_states(&self) -> crate::Result<Vec<State>> {
        crate::log_elapsed_time!(info, "get_states");
        self.backend.get_states()
    }
}
