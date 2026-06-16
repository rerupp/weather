//! The manager of a weather history archive.
//!
//!
//! This has basically been a mess since the initial implementation. When DarkSky archives
//! were converted into a history format more neutral it became worse. The current version
//! completely hides the details about the archive implementation.
//!
//! The functions mining history and content were changed to return iterators instead of
//! collections. This is different from the database implementation where it returns
//! collections.

use crate::{
    backend::filesys::{history, FilesysMetadata, WeatherFile},
    entities::{DateRange, DateRanges, History},
};
use toolslib::{fmt::commafy, stopwatch::StopWatch};

mod archive_file;
use archive_file::ArchiveFile;
pub(in crate::backend) use archive_file::{ArchiveContent, ArchiveData};

/// This is a frontend to the internal [ArchiveFile].
///
pub struct HistoryArchive {
    /// The archive file that will be accessed.
    archive: ArchiveFile,
}
impl HistoryArchive {
    /// Creates an instance of the history archive verifying the underlying
    /// archive file exists.
    ///
    /// # Arguments
    ///
    /// * `alias` is the locations unique identifier.
    /// * `archive_file` is an existing location archive file.
    ///
    pub fn open(alias: &str, archive_file: WeatherFile) -> crate::Result<Self> {
        crate::log_elapsed_time!(trace, format!("HistoryArchive({alias}) open"));
        Ok(Self { archive: ArchiveFile::open(alias, archive_file)? })
    }

    /// Creates an instance of the history archive creating the underlying
    /// archive.
    ///
    /// # Arguments
    ///
    /// * `alias` is the locations unique identifier.
    /// * `archive_file` is the weather history archive file.
    ///
    pub fn create(alias: &str, archive_file: WeatherFile) -> crate::Result<Self> {
        crate::log_elapsed_time!(trace, format!("HistoryArchive({alias}) create"));
        Ok(Self { archive: ArchiveFile::create(alias, archive_file)? })
    }

    /// Creates an iterator that returns all weather history from an archive.
    ///
    /// # Arguments
    ///
    /// * `alias` is the locations unique identifier.
    /// * `archive_file` is an existing location archive file.
    ///
    pub fn metadata_and_history_iter(
        alias: &str,
        archive_file: WeatherFile,
    ) -> crate::Result<impl Iterator<Item = (FilesysMetadata, History)>> {
        let history_archive = HistoryArchive::open(alias, archive_file)?;
        let iterator = history_archive.archive.content_iter()?;
        Ok(HistoryIterator { inner_iterator: iterator })
    }


    /// Used by the [Backend](crate::backend::Backend) to get matching history dates for a location. If a
    /// date selector is not provided all history dates will be returned.
    ///
    /// # Arguments
    ///
    /// * `selector` provides a range of history dates to match.
    ///
    pub fn dates(&self, selector: Option<&DateRange>) -> crate::Result<DateRanges> {
        let stopwatch = StopWatch::start_new();
        // DateRanges will order the dates
        let dates = self.archive.history_dates(selector, false)?;
        let date_ranges = DateRanges::new(&self.archive.lid, dates);
        log::trace!("'{}' dates: {}", &self.archive.lid, commafy(stopwatch));
        Ok(date_ranges)
    }

    /// Used by the [Backend](crate::backend::Backend) to get histories for the date range.
    ///
    /// # Arguments
    ///
    /// * `selector` provides a range of history dates to match.
    ///
    pub fn histories(&self, selector: &DateRange) -> crate::Result<impl Iterator<Item = History>> {
        let iterator = self.archive.data_iter(selector)?;
        let history_iterator = HistoryIterator { inner_iterator: iterator };
        Ok(history_iterator)
    }

    /// Used by the [Backend](crate::backend::Backend) to add histories to the location archive. Existing histories
    /// will not be overridden in the archive.
    ///
    /// # Arguments
    ///
    /// * `histories` provides the location weather history that will be added to the archive.
    ///
    pub fn add(&self, histories: &Vec<History>) -> crate::Result<Vec<FilesysMetadata>> {
        use std::{collections::HashSet, fmt::Write};
        let stopwatch = StopWatch::start_new();

        // get a collection of the history dates
        let history_dates = histories.iter().map(|history| history.date.clone()).collect::<Vec<_>>();

        // the caller must make sure that none of the new histories are duplicate
        let mut duplicate_date_checker = HashSet::new();
        for date in &history_dates {
            if !duplicate_date_checker.insert(*date) {
                super::err!("The histories update for '{}' contains duplicates", self.archive.lid)?;
            }
        }

        // remove any histories that already exist in the location storage.
        let duplicate_dates =
            self.archive.metadata_by_date(history_dates, true)?.map(|md| md.date).collect::<HashSet<_>>();
        let mut duplicates_err = String::new();
        let history_updates: Vec<&History> = match duplicate_dates.is_empty() {
            true => histories.iter().collect(),
            false => histories
                .iter()
                .filter(|history| match duplicate_dates.contains(&history.date) {
                    false => true,
                    true => {
                        write!(duplicates_err, "\n  {}", history.date).unwrap();
                        false
                    }
                })
                .collect(),
        };
        if duplicates_err.len() > 0 {
            log::warn!("These history dates already exist for Location '{}':{duplicates_err}", self.archive.lid);
        }

        // save the dates from the new histories
        let mut new_history_dates = vec![];
        let new_histories: Vec<ArchiveData> = history_updates
            .into_iter()
            .filter_map(|history| {
                new_history_dates.push(history.date);
                match history::to_bytes(history) {
                    Ok(data) => Some(ArchiveData { lid: self.archive.lid.clone(), date: history.date, data }),
                    Err(error) => {
                        log::error!("'{}' history data error on {}: {error}", self.archive.lid, history.date);
                        None
                    }
                }
            })
            .collect();
        self.archive.add_data(new_histories)?;
        log::trace!("'{}' append: {}", &self.archive.lid, commafy(stopwatch));

        // give the caller metadata for the new histories
        let new_history_metadata = self.archive.metadata_by_date(new_history_dates, false)?.collect::<Vec<_>>();
        Ok(new_history_metadata)
    }

    /// Used by the filesys::admin module to get all the metadata for a history archive.
    ///
    pub fn metadata(&self) -> crate::Result<impl Iterator<Item = FilesysMetadata>> {
        self.archive.metadata_iter(None)
    }

    /// Copy of the archive contents into the destination archive.
    ///
    /// # Arguments
    ///
    /// * `destination` is the archive contents will be copied into.
    ///
    pub fn copy(&self, destination: &HistoryArchive) -> crate::Result<()> {
        self.archive.copy_filter(&destination.archive, None)
    }

    /// Check if the archive does not contain any files.
    ///
    pub fn is_empty(&self) -> bool {
        self.archive.is_empty()
    }

    /// Get the count of history files in the archive.
    ///
    pub fn history_count(&self) -> usize {
        self.archive.history_count()
    }

    /// Get the disk file size of the archive.
    ///
    pub fn size(&self) -> u64 {
        self.archive.size()
    }
}

/// The history iterator captures the inner archive iterator for large queries such
/// as history or content.
struct HistoryIterator<I> {
    /// The inner archive iterator.
    pub inner_iterator: I,
}
/// Converts the inner archive file data into ArchiveData.
///
impl Iterator for HistoryIterator<Box<dyn Iterator<Item = ArchiveData>>> {
    type Item = History;

    fn next(&mut self) -> Option<Self::Item> {
        let mut item: Option<Self::Item> = None;
        if let Some(archive_data) = self.inner_iterator.next() {
            match archive_data.try_into() {
                Ok(history) => {
                    item.replace(history);
                }
                Err(error) => log::error!("{}", error),
            }
        }
        item
    }
}
/// Converts the inner archive file metadata and data into ArchiveContent.
///
impl Iterator for HistoryIterator<Box<dyn Iterator<Item = ArchiveContent>>> {
    type Item = (FilesysMetadata, History);

    fn next(&mut self) -> Option<Self::Item> {
        let mut item: Option<Self::Item> = None;
        if let Some(content) = self.inner_iterator.next() {
            match content.data.try_into() {
                Ok(history) => {
                    item.replace((content.metadata, history));
                }
                Err(error) => log::error!("{}", error),
            }
        }
        item
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{filesys::WeatherDir, testlib};
    use std::path::PathBuf;
    use toolslib::date_time::get_date;

    #[test]
    fn history_archive() {
        // set up the testcase
        let fixture = testlib::TestFixture::create();
        let weather_path = PathBuf::from(&fixture);
        let weather_dir = WeatherDir::new(weather_path.clone()).unwrap();
        let archive_file = weather_dir.archive("history_archive");

        // initialize the archive
        let alias = "test";
        let testcase = HistoryArchive::create(alias, archive_file).unwrap();
        assert_eq!(testcase.history_count(), 0);

        // add data to the archive
        let test_dates = DateRange::new(get_date(2025, 5, 15), get_date(2025, 5, 19));
        let history_data: Vec<History> =
            test_dates.iter().map(|date| History { alias: alias.to_string(), date, ..Default::default() }).collect();
        let appends_metadata = testcase.add(&history_data).unwrap();
        assert_eq!(appends_metadata.len(), 5);
        for metadata in appends_metadata {
            assert!(test_dates.contains(&metadata.date));
        }

        // spot check the archive
        let histories: Vec<History> = testcase.histories(&test_dates).unwrap().collect();
        assert_eq!(histories.len(), 5);
        for history in histories {
            assert!(test_dates.contains(&history.date));
        }
        let archive_file = weather_dir.archive("history_archive");
        assert!(!archive_file.with_extension(archive_file::BACKUP_EXT).exists());
        assert!(!archive_file.with_extension(archive_file::UPDATE_EXT).exists());

        // make sure you can't add histories that already exist
        let added_dates = testcase.add(&history_data).unwrap();
        assert_eq!(added_dates.len(), 0);
        assert_eq!(testcase.history_count(), 5);
    }
}
