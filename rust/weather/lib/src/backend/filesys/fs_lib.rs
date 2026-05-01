//! The [filesys] module library.

use super::err;
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
    if Locations::open(weather_dir)?.get(Some(vec![LocationFilter::alias(&location.alias)]))?.count() == 0 {
        err!("The location {location} was not found.")?;
    }

    // the history archive will make sure there are no duplicates added and issue log warnings
    let archive_file = weather_dir.archive(&location.alias);
    let archive = HistoryArchive::open(&location.alias, archive_file)?;
    let additions_metadata = archive.append(&daily_histories.histories)?;
    Ok(additions_metadata)
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
    // the document should be in location order however you can't be sure
    let mut locations = Locations::open(weather_dir)?.get(filters)?.collect::<Vec<_>>();
    locations.sort_unstable();
    Ok(locations)
}

/// The [db] module uses this function when it reloads weather history for a single location.
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

pub mod history_contents {
    //! The [db] module uses this API to get the contents of weather history archives for multiple
    //! locations.
    //!
    use super::*;
    use crate::backend::filesys::archives_iterator::ArchivesReaderCtx;
    use crate::{
        backend::filesys::{
            archives_iterator::{ArchivesIterator, ArchivesReader},
            history_archive::ArchiveMetadata,
        },
        prelude::History,
    };
    use std::sync::mpsc::Sender;

    /// The items returned by the history contents iterator.
    pub type HistoryContents = (ArchiveMetadata, History);

    /// Return an iterator that reads the contents of weather history archives. The order of
    /// returned weather history content is not guaranteed to be grouped by location.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `filters` optionally restricts which location counts will be returned.
    /// * `max_threads` limits the number of threads used (default is 16).
    ///
    pub fn get(
        weather_dir: &WeatherDir,
        filters: Option<Vec<LocationFilter>>,
        max_threads: Option<usize>,
    ) -> crate::Result<impl Iterator<Item = HistoryContents>> {
        crate::log_elapsed_time!(trace, "history_contents::get()");
        let locations = Locations::open(weather_dir)?.get(filters)?.collect::<Vec<_>>();
        let workers = max_threads.unwrap_or(16);
        Ok(ArchivesIterator::new(weather_dir, locations, workers, |sender| Box::new(Reader(sender))))
    }

    struct Reader(Sender<HistoryContents>);
    impl ArchivesReader<HistoryContents> for Reader {
        fn read_archive(&self, ctx: ArchivesReaderCtx) {
            match HistoryArchive::metadata_and_history_iter(&ctx.location.alias, ctx.file) {
                Err(error) => log::error!("Error getting history contents: {:?}", error),
                Ok(contents) => {
                    for content in contents {
                        if let Err(error) = self.0.send(content) {
                            let (_, history) = error.0;
                            log::error!("Error sending {} content for {}.", ctx.location, history.date)
                        }
                    }
                }
            }
        }
    }
}

pub mod history_counts {
    //! The [db] module uses this API to get location history counts.
    //!
    use super::*;
    use crate::backend::filesys::archives_iterator::{ArchivesIterator, ArchivesReader, ArchivesReaderCtx};
    use std::sync::mpsc::Sender;

    /// The type of data returned in the collection of history counts.
    ///
    pub type HistoryCount = (Location, usize);

    /// The API used by the that gets [Location] history counts.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `filters` optionally restricts which location counts will be returned.
    ///
    pub fn get(weather_dir: &WeatherDir, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistoryCount>> {
        crate::log_elapsed_time!(info, "history_counts::get()");
        let locations = Locations::open(weather_dir)?.get(filters)?.collect::<Vec<_>>();
        let mut history_counts =
            ArchivesIterator::new(weather_dir, locations, 16, |sender| Box::new(Reader(sender))).collect::<Vec<_>>();
        history_counts.sort_unstable_by(|(lhs, _), (rhs, _)| lhs.cmp(rhs));
        Ok(history_counts)
    }
    struct Reader(Sender<HistoryCount>);
    impl ArchivesReader<HistoryCount> for Reader {
        fn read_archive(&self, ctx: ArchivesReaderCtx) {
            match HistoryArchive::open(&ctx.location.alias, ctx.file) {
                Err(error) => log::error!("Error opening location '{}' archive: {:?}", ctx.location, error),
                Ok(archive) => {
                    if let Err(error) = self.0.send((ctx.location, archive.history_count())) {
                        let (location, _) = error.0;
                        log::error!("Failed to send history counts for {location}.");
                    }
                }
            }
        }
    }
}

pub mod history_dates {
    //! This is used by [Backend] implementations to get location history dates..
    //!
    use super::*;
    use crate::{
        backend::filesys::archives_iterator::{ArchivesIterator, ArchivesReader, ArchivesReaderCtx},
        entities::HistoryDates,
    };
    use std::sync::mpsc::Sender;

    /// The API that gets history dates for a collection of locations.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `filters` optionally restricts which location counts will be returned.
    ///
    pub fn get(
        weather_dir: &WeatherDir,
        locations: Vec<Location>,
        max_threads: usize,
    ) -> crate::Result<Vec<HistoryDates>> {
        crate::log_elapsed_time!(trace, "get_history_dates");
        let mut history_dates =
            ArchivesIterator::new(weather_dir, locations, max_threads, |sender| Box::new(Reader(sender)))
                .collect::<Vec<_>>();
        history_dates.sort_unstable_by(|lhs, rhs| lhs.location.cmp(&rhs.location));
        Ok(history_dates)
    }

    struct Reader(Sender<HistoryDates>);
    impl ArchivesReader<HistoryDates> for Reader {
        fn read_archive(&self, ctx: ArchivesReaderCtx) {
            match HistoryArchive::open(&ctx.location.alias, ctx.file) {
                Err(error) => log::error!("Could not open history archive for {}: {error}", ctx.location),
                Ok(archive) => match archive.dates(None) {
                    Err(error) => log::error!("Could not get history dates for {}: {error}", ctx.location),
                    Ok(dates) => {
                        let history_dates = HistoryDates { location: ctx.location, history_dates: dates.date_ranges };
                        if let Err(error) = self.0.send(history_dates) {
                            log::error!("Failed to send history dates for {}: {}", error.0.location, error);
                        }
                    }
                },
            }
        }
    }
}

pub mod history_summaries {
    //! Use the [ArchivesIterator] to mine the history summaries from the archives.
    //!
    use super::*;
    use crate::{
        backend::filesys::archives_iterator::{ArchivesIterator, ArchivesReader, ArchivesReaderCtx},
        entities::HistorySummaries,
    };
    use std::sync::mpsc::Sender;

    /// The API that gets a summary of weather history data for a collection of locations.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `locations` determines which history summaries to get.
    /// * `readers` provides a limit on how many readers to use.
    ///
    pub fn get(
        weather_dir: &WeatherDir,
        locations: Vec<Location>,
        readers: usize,
    ) -> crate::Result<Vec<HistorySummaries>> {
        crate::log_elapsed_time!(trace, "get_history_summaries");
        let mut history_summaries =
            ArchivesIterator::new(weather_dir, locations, readers, |sender| Box::new(Reader(sender)))
                .collect::<Vec<_>>();
        history_summaries.sort_unstable_by(|lhs, rhs| lhs.location.cmp(&rhs.location));
        Ok(history_summaries)
    }

    struct Reader(Sender<HistorySummaries>);
    impl ArchivesReader<HistorySummaries> for Reader {
        fn read_archive(&self, ctx: ArchivesReaderCtx) {
            match HistoryArchive::open(&ctx.location.alias, ctx.file) {
                Err(error) => log::error!("Could not open history archive for {}: {error}", ctx.location),
                Ok(archive) => match archive.summary() {
                    Err(error) => log::error!("Could not read history summary for {}: {error}", ctx.location),
                    Ok(history_summary) => {
                        let history_summaries = HistorySummaries {
                            location: ctx.location,
                            count: history_summary.count,
                            overall_size: history_summary.overall_size,
                            raw_size: history_summary.raw_size,
                            store_size: history_summary.compressed_size,
                        };
                        if let Err(error) = self.0.send(history_summaries) {
                            log::error!("Failed to send history summary for {}: {}", error.0.location, error);
                        }
                    }
                },
            }
        }
    }
}
