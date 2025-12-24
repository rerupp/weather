//! The filesystem objects that support implementing weather data using `ZIP` archives.

pub(crate) mod admin;

pub(in crate::backend) mod fs_lib;

mod histories_reader;

mod history;

mod history_archive;
use history_archive::HistoryArchive;

mod locations;
use locations::Locations;

mod weather_dir;
pub(crate) use weather_dir::WeatherDir;

mod weather_file;
pub(in crate::backend) use weather_file::WeatherFile;

use crate::{
    backend::Backend,
    prelude::{
        CityFilter, Configuration, DailyHistories, DateRange, HistoryDates, HistorySummaries, Location, LocationFilter,
        State,
    },
};
use std::sync::Arc;

/// Create a Locations specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("ArchiveBackend {}", format!($($arg)*)))
    }
}
use error;

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
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
        history_dates::get_history_dates(&self.weather_dir, locations, self.configuration.weather_data.max_workers)
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
        history_summaries::get_history_summaries(
            &self.weather_dir,
            locations,
            self.configuration.weather_data.max_workers,
        )
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

    /// Search US Cities for location metadata.
    ///
    /// # Arguments
    ///
    /// * `filter` identifies which cities are being searched for (default is all).
    ///
    fn search_locations(&self, _filter: CityFilter) -> crate::Result<Vec<Location>> {
        err!("Search locations is not currently available running in file system mode.")
    }

    /// Get the city state information that has been loaded.
    ///
    fn get_states(&self) -> crate::Result<Vec<State>> {
        err!("Get states is not currently available running in file system mode.")
    }
}

mod history_dates {
    //! Use the [HistoryReader] to mine history dates from the archives.
    //!
    use super::*;
    use crate::backend::filesys::histories_reader::{generate_history_reader, HistoriesReader, HistoryReader};
    use std::thread;

    /// The API that gets location history dates.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `filters` optionally restricts which location counts will be returned.
    ///
    pub fn get_history_dates(
        weather_dir: &WeatherDir,
        locations: Vec<Location>,
        max_threads: usize,
    ) -> crate::Result<Vec<HistoryDates>> {
        crate::log_elapsed_time!(trace, "get_history_dates");
        let mut history_dates =
            HistoriesReader::new(weather_dir, locations, max_threads, HistoryDatesReader::create).collect::<Vec<_>>();
        history_dates.sort_unstable_by(|lhs, rhs| lhs.location.name.cmp(&rhs.location.name));
        Ok(history_dates)
    }

    generate_history_reader!(HistoryDatesReader, HistoryDates);
    impl HistoryReader<HistoryDates> for HistoryDatesReader {
        fn read_archive(&self) {
            crate::log_elapsed_time!(format!("{:?} HistoryDatesReader", thread::current().id()));
            while let Some(item) = self.queue.take() {
                let location = item.location;
                let archive_file = item.file;
                match HistoryArchive::open(&location.alias, archive_file) {
                    Err(error) => {
                        log::error!("Could not open history archive for {}: {}", location.name, error);
                    }
                    Ok(archive) => match archive.dates(None) {
                        Err(error) => {
                            log::error!("Could not get history dates for {}: {}", location.name, error);
                        }
                        Ok(dates) => {
                            let history_dates = HistoryDates { location, history_dates: dates.date_ranges };
                            if let Err(error) = self.sender.send(history_dates) {
                                log::error!("Did not send history dates for {}: {}", error.0.location.name, error);
                            }
                        }
                    },
                }
            }
        }
    }
}

mod history_summaries {
    //! Use the [HistoryReader] to mine the history summaries from the archives.
    //!
    use super::*;
    use crate::backend::filesys::histories_reader::{generate_history_reader, HistoriesReader, HistoryReader};
    use std::thread;

    /// The API that gets location history dates.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `filters` optionally restricts which location counts will be returned.
    ///
    pub fn get_history_summaries(
        weather_dir: &WeatherDir,
        locations: Vec<Location>,
        max_threads: usize,
    ) -> crate::Result<Vec<HistorySummaries>> {
        crate::log_elapsed_time!(trace, "get_history_summaries");
        let mut history_summaries =
            HistoriesReader::new(weather_dir, locations, max_threads, HistorySummariesReader::create)
                .collect::<Vec<_>>();
        history_summaries.sort_unstable_by(|lhs, rhs| lhs.location.name.cmp(&rhs.location.name));
        Ok(history_summaries)
    }

    generate_history_reader!(HistorySummariesReader, HistorySummaries);
    impl HistoryReader<HistorySummaries> for HistorySummariesReader {
        fn read_archive(&self) {
            crate::log_elapsed_time!(format!("{:?} HistorySummariesReader", thread::current().id()));
            while let Some(item) = self.queue.take() {
                let location = item.location;
                let archive_file = item.file;
                match HistoryArchive::open(&location.alias, archive_file) {
                    Err(error) => {
                        log::error!("Could not open history archive for {}: {}", location.name, error);
                    }
                    Ok(archive) => match archive.summary() {
                        Err(error) => {
                            log::error!("Could not read history summary for {}: {}", location.name, error);
                        }
                        Ok(history_summary) => {
                            let history_summaries = HistorySummaries {
                                location,
                                count: history_summary.count,
                                overall_size: history_summary.overall_size,
                                raw_size: history_summary.raw_size,
                                store_size: history_summary.compressed_size,
                            };
                            if let Err(error) = self.sender.send(history_summaries) {
                                log::error!("Did not send history summary for {}: {}", error.0.location.name, error);
                            }
                        }
                    },
                }
            }
        }
    }
}
