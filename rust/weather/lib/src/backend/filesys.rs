//! The filesystem objects that support implementing weather data using `ZIP` archives.

pub(crate) mod admin;

mod history;

mod history_archive;
mod locations;
mod weather_dir;
mod weather_file;

pub(in crate::backend) use {
    history_archive::{ArchiveMetadata, HistoryArchive},
    locations::Locations,
    weather_dir::WeatherDir,
    weather_file::WeatherFile,
};

use super::LocationFilters;
use crate::{
    backend::{Backend, Config},
    entities::{DailyHistories, DateRange, HistoryDates, HistorySummaries, Location, State, CityFilter},
};

/// Get a [WeatherDir] instance.
pub(in crate::backend) fn create_weather_dir(dirname: &str) -> crate::Result<WeatherDir> {
    let weather_dir = if dirname.len() > 0 {
        WeatherDir::try_from(dirname)?
    } else if let Ok(env_pathname) = std::env::var("WEATHER_DATA") {
        WeatherDir::try_from(env_pathname)?
    } else {
        WeatherDir::try_from("weather_data")?
    };
    Ok(weather_dir)
}

/// Create a Locations specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("ArchiveBackend {}", format!($($arg)*)))
    }
}

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// Creates the file based data API for weather data.
///
/// # Arguments
///
/// * `config` contains the weather data configuration.
///
pub fn create_filesys_backend(config: Config) -> crate::Result<Box<dyn Backend>> {
    log::debug!("ArchiveBackend");
    let weather_dir = create_weather_dir(&config.weather_data.directory)?;
    Ok(Box::new(ArchiveBackend { config, weather_dir }))
}

/// The archive implementation of a [Backend].
struct ArchiveBackend {
    config: Config,
    /// The directory containing weather data files.rs
    weather_dir: WeatherDir,
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
}
impl Backend for ArchiveBackend {
    /// Get the backend configuration.
    ///
    fn get_config(&self) -> &Config {
        &self.config
    }

    /// Add weather data history for a location.
    ///
    /// # Arguments
    ///
    /// * `daily_histories` has the location and histories to add.
    ///
    fn add_daily_histories(&self, daily_histories: DailyHistories) -> crate::Result<usize> {
        crate::log_elapsed_time!(trace, "add_daily_histories");
        let location = &daily_histories.location;
        let archive = self.get_archive(&location.alias)?;
        let additions = archive.append(&daily_histories.histories)?;
        Ok(additions.len())
    }

    /// Returns the daily weather data history for a location.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies what location should be used.
    /// * `history_range` specifies the date range that should be used.
    ///
    fn get_daily_histories(&self, filters: LocationFilters, history_range: DateRange) -> crate::Result<DailyHistories> {
        let mut locations = self.get_locations(filters)?;
        let location = match locations.len() {
            1 => locations.pop().unwrap(),
            // 0 => Err(crate::Error::from("A location was not found."))?,
            0 => err!("a location was not found.")?,
            _ => err!("Multiple locations were found.")?,
        };
        crate::log_elapsed_time!(trace, "get_daily_histories");
        let archive = self.get_archive(&location.alias)?;
        let daily_histories = archive.histories(&history_range)?.collect();
        Ok(DailyHistories { location, histories: daily_histories })
    }

    /// Get the weather history dates for locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations.
    ///
    fn get_history_dates(&self, filters: LocationFilters) -> crate::Result<Vec<HistoryDates>> {
        let locations = self.get_locations(filters)?;
        crate::log_elapsed_time!(trace, "get_history_dates");
        let mut history_dates = Vec::with_capacity(locations.len());
        for location in locations {
            let archive = self.get_archive(&location.alias)?;
            let dates = archive.dates(None)?;
            history_dates.push(HistoryDates { location, history_dates: dates.date_ranges })
        }
        Ok(history_dates)
    }

    /// Get the summary metrics of a locations weather data.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations that should be used.
    ///
    fn get_history_summaries(&self, filters: LocationFilters) -> crate::Result<Vec<HistorySummaries>> {
        let locations = self.get_locations(filters)?;
        crate::log_elapsed_time!(trace, "get_history_summaries");
        let mut history_summaries = Vec::with_capacity(locations.len());
        for location in locations {
            let archive = self.get_archive(&location.alias)?;
            let summary = archive.summary()?;
            history_summaries.push(HistorySummaries {
                location,
                count: summary.count,
                overall_size: summary.overall_size,
                raw_size: summary.raw_size,
                store_size: summary.compressed_size,
            });
        }
        Ok(history_summaries)
    }

    /// Get the metadata for weather locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations of interest.
    ///
    fn get_locations(&self, filters: LocationFilters) -> crate::Result<Vec<Location>> {
        crate::log_elapsed_time!(trace, "get_locations");
        let locations = match filters.is_empty() {
            true => Locations::open(&self.weather_dir)?.get()?.collect(),
            false => Locations::open(&self.weather_dir)?.find(filters)?.collect(),
        };
        Ok(locations)
    }

    /// Add a new weather location.
    ///
    /// # Arguments
    ///
    /// * `location` is the location that will be added.
    ///
    fn add_location(&self, location: Location) -> crate::Result<()> {
        crate::log_elapsed_time!(trace, "add_location");
        Locations::open(&self.weather_dir)?.add(location)?;
        Ok(())
    }

    /// Search US Cities for location metadata.
    ///
    /// # Arguments
    ///
    /// * `filter` identifies which cities are being searched for (default is all).
    ///
    fn search_locations(&self, filter: CityFilter) -> crate::Result<Vec<Location>> {
        crate::log_elapsed_time!(trace, "search_locations");
        use crate::backend::db::sqlite as db;
        match !db::us_cities::exists(&self.weather_dir) {
            true => Ok(vec![]),
            false => db::us_cities::get_cities(&db::us_cities::open(&self.weather_dir)?, filter),
        }
    }

    /// Get the city state information that has been loaded.
    ///
    fn get_states(&self) -> crate::Result<Vec<State>> {
        crate::log_elapsed_time!(trace, "search_locations");
        use crate::backend::db::sqlite as db;
        match !db::us_cities::exists(&self.weather_dir) {
            true => Ok(vec![]),
            false => db::us_cities::get_states(&db::us_cities::open(&self.weather_dir)?),
        }
    }
}
