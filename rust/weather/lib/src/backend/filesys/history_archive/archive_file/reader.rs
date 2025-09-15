//! The archive file reader.
//!
use super::{
    archive,
    iterators::{ArchiveContentIterator, ArchiveDataIterator, ArchiveMetadataIterator},
};
use crate::{backend::filesys::WeatherFile, entities::DateRange};
use chrono::NaiveDate;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Seek},
};
use zip::{ZipArchive, ZipWriter};

/// The [ArchiveReader] error builder.
macro_rules! error {
    ($id:expr, $message:expr) => {
        crate::Error::from(format!("'{}' ArchiveReader {}", $id, $message))
    };
}

/// The archive reader provides the API to return weather the history dates, metadata, and data within
/// a zip archive. Its use should be short-lived because it holds onto a ZipArchive instance to get
/// file contents.
///
#[derive(Debug)]
pub struct ArchiveReader<R: Read + Seek> {
    /// The unique identifier for a location.
    pub lid: String,
    /// The zip archive reader
    archive: ZipArchive<R>,
}
impl<R: Read + Seek> ArchiveReader<R> {
    /// Get the dates of all history files in the archive.
    ///
    pub fn dates(&self) -> crate::Result<Vec<NaiveDate>> {
        let mut dates: Vec<NaiveDate> = Vec::new();
        for filename in self.archive.file_names() {
            dates.push(archive::filename_to_date(filename)?);
        }
        Ok(dates)
    }

    /// Get the dates of history files covered by the date range.
    ///
    /// # Arguments
    ///
    /// * `date_range` identifies which history dates should be returned.
    ///
    pub fn dates_from_date_range(&self, date_range: &DateRange) -> crate::Result<Vec<NaiveDate>> {
        let mut dates: Vec<NaiveDate> = Vec::new();
        for filename in self.archive.file_names() {
            let date = archive::filename_to_date(filename)?;
            if date_range.covers(&date) {
                dates.push(date)
            }
        }
        Ok(dates)
    }

    /// Check if history exists.
    ///
    /// # Arguments
    ///
    /// * `date` is the history date that will be checked.
    ///
    pub fn contains(&self, date: &NaiveDate) -> bool {
        let filename = archive::date_to_filename(&self.lid, date);
        self.archive.index_for_name(&filename).is_some()
    }

    /// Get history metadata for a collection of dates.
    ///
    /// # Arguments
    ///
    /// * `dates` identifies which history metadata will be returned.
    ///
    pub fn metadata_by_date(self, dates: Vec<NaiveDate>) -> crate::Result<ArchiveMetadataIterator<R>> {
        Ok(ArchiveMetadataIterator::new(&self.lid, self.archive, dates))
    }

    /// Get history data for a collection of dates.
    ///
    /// # Arguments
    ///
    /// * `dates` identifies which history metadata will be returned.
    ///
    pub fn data_by_date(self, dates: Vec<NaiveDate>) -> crate::Result<ArchiveDataIterator<R>> {
        Ok(ArchiveDataIterator::new(&self.lid, self.archive, dates))
    }

    /// Get history data for a collection of dates.
    ///
    /// # Arguments
    ///
    /// * `dates` identifies which history metadata will be returned.
    ///
    pub fn content_by_date(self, dates: Vec<NaiveDate>) -> crate::Result<ArchiveContentIterator<R>> {
        Ok(ArchiveContentIterator::new(&self.lid, self.archive, dates))
    }
}
/// Implement the archive reader using a buffered file reader.
///
impl ArchiveReader<BufReader<File>> {
    /// Open the weather history archive file.
    ///
    /// An error will be returned if the archive does not exist or is not valid.
    ///
    /// # Arguments
    ///
    /// * `lid` is the location identifier.
    /// * `file` is the archive containing of weather data.
    ///
    pub fn open(lid: &str, file: &WeatherFile) -> crate::Result<Self> {
        Ok(Self { lid: lid.to_string(), archive: archive::open(file)? })
    }

    /// Creates a weather data archive.
    ///
    /// An error will be returned if the archive exists or there are problems trying to create it.
    ///
    /// # Arguments
    ///
    /// * `lid` is the location identifier.
    /// * `file` is the container of weather data.
    ///
    pub fn create(lid: &str, file: &WeatherFile) -> crate::Result<Self> {
        // touch the file
        if let Err(open_error) = OpenOptions::new().create_new(true).write(true).open(&file.to_string()) {
            Err(error!(lid, format!("did not create history file {}: {:?}", file, open_error)))
        // create the archive
        } else if let Err(zip_error) = ZipWriter::new(file.writer()?).finish() {
            Err(error!(lid, format!("did not create history archive {}: {:?}", file, zip_error)))
        } else {
            Self::open(lid, file)
        }
    }
}
