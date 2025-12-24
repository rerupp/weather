//! The manager of weather history archives.
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
    backend::filesys::{history, WeatherFile},
    entities::{DateRange, DateRanges, History, HistorySummary},
};
use toolslib::{fmt::commafy, stopwatch::StopWatch};

mod archive_file;
use archive_file::ArchiveFile;
pub(in crate::backend) use archive_file::{ArchiveContent, ArchiveData, ArchiveMetadata};
use chrono::NaiveDate;

pub struct HistoryArchive {
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
    ) -> crate::Result<impl Iterator<Item = (ArchiveMetadata, History)>> {
        let history_archive = HistoryArchive::open(alias, archive_file)?;
        let iterator = history_archive.archive.content_iter()?;
        Ok(HistoryIterator { inner_iterator: iterator })
    }

    /// Used by the [Backend] to get a summary of the history information for a location.
    ///
    pub fn summary(&self) -> crate::Result<HistorySummary> {
        let stopwatch = StopWatch::start_new();
        let mut files: usize = 0;
        let mut size: u64 = 0;
        let mut compressed_size: u64 = 0;
        self.archive.metadata_iter(None)?.for_each(|file_metadata| {
            files += 1;
            size += file_metadata.size;
            compressed_size += file_metadata.compressed_size;
        });
        let history_summary = HistorySummary {
            location_id: self.archive.lid.clone(),
            count: files,
            overall_size: Some(self.archive.size() as usize),
            raw_size: Some(size as usize),
            compressed_size: Some(compressed_size as usize),
        };
        log::trace!("'{}' summary: {}", &self.archive.lid, commafy(stopwatch));
        Ok(history_summary)
    }

    /// Used by the [Backend] to get matching history dates for a location. If a
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

    /// Used by the [Backend] to get histories for the date range.
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

    /// Used by the [Backend] to add histories to the location archive. Existing histories
    /// will not be overridden in the archive.
    ///
    /// # Arguments
    ///
    /// * `histories` provides the location weather history that will be added to the archive.
    ///
    pub fn append(&self, histories: &Vec<History>) -> crate::Result<Vec<ArchiveMetadata>> {
        let stopwatch = StopWatch::start_new();
        // find histories dates that already exist
        let append_dates = histories.iter().map(|history| history.date.clone()).collect::<Vec<_>>();
        let existing_dates = self.archive.metadata_by_date(append_dates, true)?.map(|md| md.date).collect::<Vec<_>>();

        // filter out histories that already exist
        let mut duplicate_dates: Vec<NaiveDate> = vec![];
        let mut updates: Vec<ArchiveData> = histories
            .iter()
            .filter_map(|history| {
                // remember the duplicate history dates
                let history_date = history.date.clone();
                if existing_dates.binary_search_by(|date| date.cmp(&history.date)).is_ok() {
                    duplicate_dates.push(history_date);
                    None
                } else {
                    match history::to_bytes(history) {
                        Ok(data) => Some(ArchiveData { lid: self.archive.lid.clone(), date: history_date, data }),
                        Err(error) => {
                            log::error!("'{}' history data error on {}: {}", self.archive.lid, history_date, error);
                            None
                        }
                    }
                }
            })
            .collect();

        // check if there are histories in the update that already exist
        if duplicate_dates.len() > 0 {
            let duplicates: Vec<String> = duplicate_dates.iter().map(|d| d.to_string()).collect();
            log::warn!("These histories already exist for {}: {}", self.archive.lid, duplicates.join(", "))
        }

        // check if the updates have duplicates
        duplicate_dates.clear();
        updates.sort_by(|lhs, rhs| lhs.date.cmp(&rhs.date));
        updates.dedup_by(|lhs, rhs| {
            let is_duplicate = lhs.date == rhs.date;
            if is_duplicate {
                duplicate_dates.push(lhs.date.clone());
            }
            is_duplicate
        });
        if duplicate_dates.len() > 0 {
            let duplicates = duplicate_dates.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ");
            log::warn!("'{}' history update had these duplicate dates: [{duplicates}]", self.archive.lid)
        }

        // finally update the archive
        let append_dates = updates.iter().map(|file_data| file_data.date.clone()).collect::<Vec<_>>();
        self.archive.add_data(updates)?;
        log::trace!("'{}' append: {}", &self.archive.lid, commafy(stopwatch));

        // let existing_dates = self.archive.metadata_by_date(append_dates, true)?.map(|md| md.date).collect::<Vec<_>>();
        let appends_metadata = self.archive.metadata_by_date(append_dates, true)?.collect::<Vec<_>>();
        Ok(appends_metadata)
    }

    /// Used by the filesys::admin module to get all the metadata for a history archive.
    pub fn metadata(&self) -> crate::Result<impl Iterator<Item = ArchiveMetadata>> {
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
    type Item = (ArchiveMetadata, History);

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
        assert_eq!(testcase.summary().unwrap().count, 0);

        // add data to the archive
        let test_dates = DateRange::new(get_date(2025, 5, 15), get_date(2025, 5, 19));
        let history_data: Vec<History> =
            test_dates.iter().map(|date| History { alias: alias.to_string(), date, ..Default::default() }).collect();
        let appends_metadata = testcase.append(&history_data).unwrap();
        assert_eq!(appends_metadata.len(), 5);
        for metadata in appends_metadata {
            assert!(test_dates.contains(&metadata.date));
        }

        // spot check the archive
        assert_eq!(testcase.summary().unwrap().count, 5);
        let histories: Vec<History> = testcase.histories(&test_dates).unwrap().collect();
        assert_eq!(histories.len(), 5);
        for history in histories {
            assert!(test_dates.contains(&history.date));
        }
        let archive_file = weather_dir.archive("history_archive");
        assert!(!archive_file.with_extension(archive_file::BACKUP_EXT).exists());
        assert!(!archive_file.with_extension(archive_file::UPDATE_EXT).exists());

        // make sure you can't add histories that already exist
        let added_dates = testcase.append(&history_data).unwrap();
        assert_eq!(added_dates.len(), 0);
        assert_eq!(testcase.summary().unwrap().count, 5);
    }
}
