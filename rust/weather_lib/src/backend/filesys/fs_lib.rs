//! This library is used internally to add, update, delete, and query weather history.
//!
//! This library provides utilities that can be used by [backend](crate::backend) implementations.
//! All updates to weather history in the filesystem are done through this library.

use crate::{
    backend::{
        filesys::{history_archive::HistoryArchive, FilesysMetadata, Locations},
        WeatherDir,
    },
    entities::{DailyHistories, History, Location, LocationFilter},
};

pub mod daily_history {
    use super::*;

    pub fn add(weather_dir: &WeatherDir, daily_histories: &mut DailyHistories) -> crate::Result<Vec<FilesysMetadata>> {
        // discard duplicate histories
        daily_histories.histories.sort_by(|lhs, rhs| lhs.date.cmp(&rhs.date));
        daily_histories.histories.dedup_by(|lhs, rhs| match lhs.date == rhs.date {
            false => false,
            true => {
                log::error!("Add daily history: {} has duplicate history on {}.", daily_histories.location, lhs.date);
                true
            }
        });

        // add the histories to the history archive
        let archive_file = weather_dir.archive(&daily_histories.location.alias);
        let archive = HistoryArchive::open(&daily_histories.location.alias, archive_file)?;
        archive.add(&daily_histories.histories)
    }
}

/// This is used by [Backend](crate::backend::Backend) implementations to add a location to the filesystem.
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

/// This is used by [Backend](crate::backend::Backend) implementations to delete a location.
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

/// This is used by [Backend](crate::backend::Backend) implementations to update the properties of a location.
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

/// This is used by [Backend](crate::backend::Backend) implementations to get location metadata.
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

/// The [db](crate::backend::db) module uses this function when it reloads weather history for a single location.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
/// * `alias` is the location alias name.
///
pub fn history_contents(
    weather_dir: &WeatherDir,
    alias: &str,
) -> crate::Result<impl Iterator<Item = (FilesysMetadata, History)>> {
    HistoryArchive::metadata_and_history_iter(alias, weather_dir.archive(alias))
}

pub mod history_contents {
    //! The [db](crate::backend::db) module uses this API to get the contents of weather history archives
    //! for multiple locations.
    //!
    use super::*;
    use crate::backend::filesys::archives_iterator::ArchivesReaderCtx;
    use crate::{
        backend::filesys::archives_iterator::{ArchivesIterator, ArchivesReader},
        prelude::History,
    };
    use std::sync::mpsc::Sender;

    /// The items returned by the history contents iterator.
    pub type HistoryContents = (FilesysMetadata, History);

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
    //! The [db](crate::backend::db) module uses this API to get location history counts.
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
    //! This is used by [Backend](crate::backend::Backend) implementations to get location history dates.
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

pub mod history_metadata {
    //! Use the [ArchivesIterator] to mine the metadata from the weather history data archives.
    //!
    use super::*;
    use crate::{
        backend::filesys::archives_iterator::{ArchivesIterator, ArchivesReader, ArchivesReaderCtx},
        entities::HistorySummary,
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
    ) -> crate::Result<Vec<HistorySummary>> {
        crate::log_elapsed_time!(trace, "get_history_summaries");
        let mut history_summaries =
            ArchivesIterator::new(weather_dir, locations, readers, |sender| Box::new(Reader(sender)))
                .collect::<Vec<_>>();
        history_summaries.sort_unstable_by(|lhs, rhs| lhs.location.cmp(&rhs.location));
        Ok(history_summaries)
    }

    struct Reader(Sender<HistorySummary>);
    impl ArchivesReader<HistorySummary> for Reader {
        fn read_archive(&self, ctx: ArchivesReaderCtx) {
            match HistoryArchive::open(&ctx.location.alias, ctx.file) {
                Err(error) => log::error!("Could not open history archive for {}: {error}", ctx.location),
                Ok(archive) => match archive.metadata() {
                    Err(error) => log::error!("Could not get Archive metadata for {}: {error:?}", ctx.location),
                    Ok(iter) => {
                        let mut history_summary = HistorySummary { location: ctx.location, ..Default::default() };
                        iter.for_each(|filesys_metadata| {
                            history_summary.days += 1;
                            history_summary.fs_history_summary.uncompressed_size += filesys_metadata.uncompressed_size;
                            history_summary.fs_history_summary.compressed_size += filesys_metadata.compressed_size;
                            history_summary.fs_history_summary.data_size += filesys_metadata.data_size;
                        });
                        history_summary.fs_history_summary.archive_size = archive.size();
                        if let Err(error) = self.0.send(history_summary) {
                            log::error!("Failed to send history summary for {}: {}", error.0.location, error);
                        }
                    }
                },
            }
        }
    }
}

pub mod history_filesys_details {
    //! Use the [ArchivesIterator] to mine details about weather history archives.
    //!

    use super::*;
    use crate::{
        admin::entities::LocationDetails,
        backend::filesys::archives_iterator::{ArchivesIterator, ArchivesReader, ArchivesReaderCtx},
    };
    use std::sync::mpsc::Sender;

    /// The API that mines the [LocationDetails] and archive size for locations weather history.
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
        readers: Option<usize>,
    ) -> crate::Result<Vec<(LocationDetails, usize)>> {
        crate::log_elapsed_time!(trace, "history_filesys_details");
        let mut readers = readers.unwrap_or(16);
        readers = std::cmp::min(readers, locations.len());
        let mut filesys_details =
            ArchivesIterator::new(weather_dir, locations, readers, |sender| Box::new(Reader(sender)))
                .collect::<Vec<_>>();
        filesys_details.sort_unstable_by(|lhs, rhs| lhs.0.alias.cmp(&rhs.0.alias));
        Ok(filesys_details)
    }

    struct Reader(Sender<(LocationDetails, usize)>);
    impl ArchivesReader<(LocationDetails, usize)> for Reader {
        fn read_archive(&self, ctx: ArchivesReaderCtx) {
            // capture the file size before getting the history archive
            let archive_size = ctx.file.size() as usize;
            match HistoryArchive::open(&ctx.location.alias, ctx.file) {
                Err(error) => log::error!("Could not open history archive for {}: {error}", ctx.location),
                Ok(archive) => match archive.metadata() {
                    Err(error) => log::error!("Failed to get metadata iterator for {}: {error}", ctx.location),
                    Ok(metadata_iter) => {
                        let mut histories: usize = 0;
                        let compressed_size = metadata_iter
                            .map(|metadata| {
                                histories += 1;
                                metadata.compressed_size as usize
                            })
                            .sum::<usize>();
                        let location_details =
                            LocationDetails { alias: ctx.location.alias, size: compressed_size, histories };
                        if let Err(error) = self.0.send((location_details, archive_size)) {
                            let (location_details, _) = error.0;
                            log::error!("Error sending {} archive details.", location_details.alias);
                        }
                    }
                },
            }
        }
    }
}
