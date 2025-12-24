//! The [filesys] module library.

use super::{err, error};
use crate::{
    backend::{
        filesys::{
            history_archive::{ArchiveMetadata, HistoryArchive},
            Locations,
        },
        WeatherDir,
    },
    entities::{DailyHistories, History, Location, LocationFilter},
};

/// Add weather history to a locations archive.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
/// * `daily_histories` contains the location weather history that will be added.
///
pub fn add_daily_history(
    weather_dir: &WeatherDir,
    daily_histories: &DailyHistories,
) -> crate::Result<Vec<ArchiveMetadata>> {
    // make sure the location exists before adding any histories
    let location = &daily_histories.location;
    if Locations::open(weather_dir)?.get(Some(vec![LocationFilter::name(&location.alias)]))?.count() == 0 {
        err!("The location {} ({}) was not found.", location.name, location.alias)?;
    }

    // the history archive will make sure there are no duplicates added and issue log warnings
    let archive_file = weather_dir.archive(&location.alias);
    let archive = HistoryArchive::open(&location.alias, archive_file)?;
    let additions_metadata = archive.append(&daily_histories.histories)?;
    Ok(additions_metadata)
}

/// The [db] module uses this function when it reloads weather history for a location.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
/// * `alias` is the location alias name.
///
pub fn history_contents(
    weather_dir: &WeatherDir,
    alias: &str,
) -> crate::Result<impl Iterator<Item = (ArchiveMetadata, History)>> {
    HistoryArchive::metadata_and_history_iter(alias, weather_dir.archive(alias))
}

pub use history_counts::get_history_counts;
mod history_counts {
    //! The [db] module uses this API to get location history counts.
    //!
    use super::*;
    use crate::backend::filesys::histories_reader::{generate_history_reader, HistoriesReader, HistoryReader};
    use std::thread;

    /// The API that gets [Location] history counts.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `filters` optionally restricts which location counts will be returned.
    ///
    pub fn get_history_counts(
        weather_dir: &WeatherDir,
        filters: Option<Vec<LocationFilter>>,
    ) -> crate::Result<Vec<(Location, usize)>> {
        crate::log_elapsed_time!(trace, "get_history_counts");
        let locations = Locations::open(weather_dir)?.get(filters)?.collect::<Vec<_>>();
        let mut history_counts =
            HistoriesReader::new(weather_dir, locations, 16, HistoryCounter::create).collect::<Vec<_>>();
        history_counts.sort_unstable_by(|lhs, rhs| lhs.location.name.cmp(&rhs.location.name));
        Ok(history_counts.into_iter().map(|history_count| (history_count.location, history_count.count)).collect())
    }

    /// The history data mined by the reader.
    ///
    struct HistoryCount {
        /// The location properties.
        pub location: Location,
        /// The count of daily weather histories.
        pub count: usize,
    }

    generate_history_reader!(HistoryCounter, HistoryCount);
    impl HistoryReader<HistoryCount> for HistoryCounter {
        fn read_archive(&self) {
            crate::log_elapsed_time!(format!("{:?} read_archive", thread::current().id()));
            while let Some(item) = self.queue.take() {
                log::trace!("{:?} counting {}", thread::current().id(), item.location.name);
                let location = item.location;
                let archive_file = item.file;
                match HistoryArchive::open(&location.alias, archive_file) {
                    Err(error) => {
                        log::error!("Error opening archive: {:?}", error);
                    }
                    Ok(archive) => {
                        let data = HistoryCount { location, count: archive.history_count() };
                        if let Err(error) = self.sender.send(data) {
                            log::error!("Failed to send history counts for {}.", error.0.location.name);
                        }
                    }
                }
            }
        }
    }
}

pub use history_contents::get_history_contents;
mod history_contents {
    //! The [db] module uses this API to get the contents of weather history archives.
    //!
    use super::*;
    use crate::{
        backend::filesys::{
            histories_reader::{generate_history_reader, HistoriesReader, HistoryReader},
            history_archive::ArchiveMetadata,
        },
        prelude::History,
    };
    use std::thread;

    /// The archive contents.
    ///
    type ArchiveContents = (ArchiveMetadata, History);

    /// Get an iterator that reads the contents of weather history archives. The order of
    /// returned weather history content is not guaranteed to be grouped by location.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `filters` optionally restricts which location counts will be returned.
    /// * `max_threads` limits the number of threads used (default is 16).
    ///
    pub fn get_history_contents(
        weather_dir: &WeatherDir,
        filters: Option<Vec<LocationFilter>>,
        max_threads: Option<usize>,
    ) -> crate::Result<impl Iterator<Item = ArchiveContents>> {
        crate::log_elapsed_time!(trace, "get_history_contents");
        let locations = Locations::open(weather_dir)?.get(filters)?.collect::<Vec<_>>();
        let max_threads = max_threads.unwrap_or(16);
        Ok(HistoriesReader::new(weather_dir, locations, max_threads, HistoryContentsReader::create))
    }

    generate_history_reader!(HistoryContentsReader, ArchiveContents);
    impl HistoryReader<ArchiveContents> for HistoryContentsReader {
        fn read_archive(&self) {
            crate::log_elapsed_time!(format!("{:?} HistoryReader read_archive", thread::current().id()));
            while let Some(item) = self.queue.take() {
                log::trace!("{:?} mining histories for {}", thread::current().id(), item.location.name);
                let location = item.location;
                let archive_file = item.file;
                match HistoryArchive::metadata_and_history_iter(&location.alias, archive_file) {
                    Err(error) => {
                        log::error!("Error getting history contents: {:?}", error);
                    }
                    Ok(contents) => {
                        for content in contents {
                            if let Err(error) = self.sender.send(content) {
                                let (_, history) = error.0;
                                log::error!("Error sending {} content for {}.", location.name, history.date)
                            }
                        }
                    }
                }
            }
        }
    }
}

/// This is used by [Backend] implementations to add a location to the filesystem.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
/// * `location` is what will be added to the store.
///
#[inline]
pub fn add_location(weather_dir: &WeatherDir, location: Location) -> crate::Result<Location> {
    Locations::open(weather_dir)?.add(location)
}

/// This is used by [Backend] implementations to delete a location.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
/// * `alias` identifies which location will be removed from the store.
///
#[inline]
pub fn delete_location(weather_dir: &WeatherDir, alias: &str) -> crate::Result<bool> {
    Locations::open(weather_dir)?.delete(alias)
}

/// This is used by [Backend] implementations to update the properties of a location.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
/// * `location` identifies the location and contains the new properties.
///
#[inline]
pub fn update_location(weather_dir: &WeatherDir, location: Location) -> crate::Result<Option<Location>> {
    Locations::open(weather_dir)?.update(location)
}

/// This is used by [Backend] implementations to get location metadata.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
/// * `filters` can be used to restrict the location metadata being returned.
///
#[inline]
pub fn get_locations(weather_dir: &WeatherDir, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<Location>> {
    Ok(Locations::open(weather_dir)?.get(filters)?.collect::<Vec<_>>())
}
