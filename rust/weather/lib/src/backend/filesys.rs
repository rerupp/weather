//! The filesystem objects that support implementing weather data using `ZIP` archives.

pub(crate) mod admin;

pub(in crate::backend) mod fs_lib;

mod history;

mod history_archive;
use history_archive::HistoryArchive;

mod locations;
use locations::Locations;

mod weather_dir;
pub(crate) use weather_dir::WeatherDir;

mod archives_iterator;
mod weather_file;

pub(in crate::backend) use weather_file::WeatherFile;

use crate::{
    backend::Backend,
    prelude::{Configuration, DailyHistories, DateRange, HistoryDates, HistorySummaries, Location, LocationFilter},
};
use std::sync::Arc;

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error::from(format!("ArchiveBackend {}", format!($($arg)*))))
    };
}
use err;

/// Creates the file based data API for weather data.
///
/// # Arguments
///
/// * `config` contains the weather data configuration.
///
pub fn create_filesys_backend(configuration: Arc<Configuration>) -> crate::Result<Box<dyn Backend>> {
    log::debug!("ArchiveBackend");
    let weather_dir = WeatherDir::try_from(&configuration)?;
    Ok(Box::new(ArchiveBackend { weather_dir, configuration }))
}

/// The archive implementation of a [Backend].
struct ArchiveBackend {
    /// The directory containing weather history files.
    weather_dir: WeatherDir,
    /// The weather data configuration
    configuration: Arc<Configuration>,
}
impl ArchiveBackend {
    /// Used internally to get the archive manager for some location.
    ///
    /// # Arguments
    ///
    /// * `alias` is the location identifier.
    ///
    fn get_archive(&self, alias: &str) -> crate::Result<HistoryArchive> {
        let weather_file = self.weather_dir.archive(alias);
        HistoryArchive::open(alias, weather_file)
    }

    /// Get a location.
    ///
    /// # Arguments
    ///
    /// * `filter` identifies what location to get.
    ///
    fn get_location(&self, filter: LocationFilter) -> crate::Result<Option<Location>> {
        let mut locations = fs_lib::get_locations(&self.weather_dir, Some(vec![filter]))?;
        match locations.len() {
            0 => Ok(None),
            1 => Ok(locations.pop()),
            _ => err!("Multiple locations were found.")?,
        }
    }
}
impl Backend for ArchiveBackend {
    /// Add weather data history for a location.
    ///
    /// # Arguments
    ///
    /// * `daily_histories` has the location and histories to add.
    ///
    fn add_daily_histories(&self, daily_histories: DailyHistories) -> crate::Result<usize> {
        crate::log_elapsed_time!(trace, "add_daily_histories");
        let additions = fs_lib::add_daily_history(&self.weather_dir, &daily_histories)?;
        Ok(additions.len())
    }

    /// Returns the daily weather data history for a location.
    ///
    /// # Arguments
    ///
    /// * `filter` identifies what location should be used.
    /// * `history_range` specifies the date range that should be used.
    ///
    fn get_daily_histories(&self, filter: LocationFilter, history_range: DateRange) -> crate::Result<DailyHistories> {
        crate::log_elapsed_time!(trace, "get_daily_histories");
        match self.get_location(filter)? {
            None => err!("A location was not found."),
            Some(location) => {
                let archive = self.get_archive(&location.alias)?;
                let histories = archive.histories(&history_range)?.collect();
                Ok(DailyHistories { location, histories })
            }
        }
    }

    /// Get the weather history dates for locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations.
    ///
    fn get_history_dates(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistoryDates>> {
        crate::log_elapsed_time!(trace, "get_history_dates");
        let locations = fs_lib::get_locations(&self.weather_dir, filters)?;
        fs_lib::history_dates::get(&self.weather_dir, locations, self.configuration.weather_data.max_workers)
    }

    /// Get the summary metrics of a locations weather data.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations that should be used.
    ///
    fn get_history_summaries(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistorySummaries>> {
        crate::log_elapsed_time!(trace, "get_history_summaries");
        let locations = fs_lib::get_locations(&self.weather_dir, filters)?;
        fs_lib::history_summaries::get(&self.weather_dir, locations, self.configuration.weather_data.max_workers)
    }

    /// Get the metadata for weather locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations of interest.
    ///
    fn get_locations(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<Location>> {
        crate::log_elapsed_time!(trace, "get_locations");
        fs_lib::get_locations(&self.weather_dir, filters)
    }

    /// Add a new weather location.
    ///
    /// # Arguments
    ///
    /// * `location` is the location that will be added.
    ///
    fn add_location(&self, location: Location) -> crate::Result<()> {
        crate::log_elapsed_time!(trace, "add_location");
        fs_lib::add_location(&self.weather_dir, location)?;
        Ok(())
    }

    /// Delete a location.
    ///
    /// # Arguments
    ///
    /// * `filter` is used to get the location alias name.
    ///
    fn delete_location(&self, filter: LocationFilter) -> crate::Result<()> {
        match self.get_location(filter)? {
            None => err!("Did not find a location to delete."),
            Some(location) => {
                fs_lib::delete_location(&self.weather_dir, &location.alias)?;
                Ok(())
            }
        }
    }

    /// Update a locations properties
    ///
    /// # Arguments
    ///
    /// * `location` identifies the location and contains the new property values.
    ///
    fn update_location(&self, location: Location) -> crate::Result<bool> {
        Ok(fs_lib::update_location(&self.weather_dir, location)?.is_some())
    }
}
