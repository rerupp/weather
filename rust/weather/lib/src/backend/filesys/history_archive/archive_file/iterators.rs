//! The various history archive iterators are located here.

use super::{archive, ArchiveContent, ArchiveData, ArchiveMetadata};
use chrono::NaiveDate;
use std::io::{Read, Seek};
use zip::{read::ZipFile, ZipArchive};

/// All iterators need to walk the archive looking for history files. This facade
/// hides all the gory details about how that is done.
struct ArchiveIterator<R: Read + Seek> {
    /// The location lid.
    lid: String,
    /// The actual zip archive.
    archive: ZipArchive<R>,
    /// The history dates to grab.
    dates: Vec<NaiveDate>,
    /// The current date index.
    index: usize,
    /// The size of the date collection.
    max_index: usize,
}
impl<R: Read + Seek> ArchiveIterator<R> {
    /// Create a new instance of the iterator.
    ///
    /// # Arguments
    ///
    /// * `lid` is the location alias.
    /// * `archive` is the zip archive that will be used.
    /// * `dates` selects which history files to iterate over.
    ///
    pub fn new(lid: &str, archive: ZipArchive<R>, dates: Vec<NaiveDate>) -> Self {
        let max_index = dates.len();
        Self { lid: lid.into(), archive, dates, index: 0, max_index }
    }

    /// There should never be an error getting contents unless something is pretty AFU.
    /// Iteration will end if there is an error and the reason will be written to the
    /// log file.
    ///
    pub fn try_next(&mut self) -> Option<(NaiveDate, ZipFile<R>)> {
        let mut item = None;
        if self.index < self.max_index {
            // get the next date
            let date = self.dates[self.index];
            self.index += 1;

            // get the history file for the date
            match self.archive.by_name(&archive::date_to_filename(&self.lid, &date)) {
                Ok(zipfile) => {
                    item.replace((date, zipfile));
                }
                Err(error) => {
                    log::error!("'{}' failed to open history on {}: {:?}", self.lid, date, error);
                    self.index = usize::MAX;
                }
            }
        }
        item
    }
}

/// The iterator over archive metadata.
/// 
pub struct ArchiveMetadataIterator<R: Read + Seek> {
    /// The archive helper that return a history ZipFile
    archive: ArchiveIterator<R>,
}
impl<R: Read + Seek> ArchiveMetadataIterator<R> {
    /// Create a new instance of the metadata iterator.
    /// 
    /// # Arguments
    /// 
    /// * `lid` is the location alias name.
    /// * `archive` is the zip archive that will be used by the iterator.
    /// * `dates` identifies what history dates will be used.
    /// 
    pub fn new(lid: &str, archive: ZipArchive<R>, dates: Vec<NaiveDate>) -> Self {
        Self { archive: ArchiveIterator::new(lid, archive, dates) }
    }
}
/// Allow the collection archive metadata to be returned as an iterator.
/// 
impl<R: Read + Seek> Iterator for ArchiveMetadataIterator<R> {
    type Item = ArchiveMetadata;

    fn next(&mut self) -> Option<Self::Item> {
        let mut archive_metadata: Option<ArchiveMetadata> = None;
        if let Some((date, zipfile)) = self.archive.try_next() {
            archive_metadata.replace(ArchiveMetadata::new(&date, &zipfile));
        }
        archive_metadata
    }
}

/// The iterator over archive file data.
pub struct ArchiveDataIterator<R: Read + Seek> {
    lid: String,
    /// The archive helper that return a history ZipFile
    archive: ArchiveIterator<R>,
    /// This allows the iterator to turn itself off.
    get_next: bool,
}
impl<R: Read + Seek> ArchiveDataIterator<R> {
    /// Create a new instance of the archive data iterator.
    /// 
    /// # Arguments
    ///
    /// * `lid` is the location alias name.
    /// * `archive` is the zip archive that will be used by the iterator.
    /// * `dates` identifies what history dates will be used.
    ///
    pub fn new(lid: &str, archive: ZipArchive<R>, dates: Vec<NaiveDate>) -> Self {
        Self { lid: lid.into(), archive: ArchiveIterator::new(lid, archive, dates), get_next: true }
    }
}
/// Allow the collection archive data to be returned as an iterator.
///
impl<R: Read + Seek> Iterator for ArchiveDataIterator<R> {
    type Item = ArchiveData;

    fn next(&mut self) -> Option<Self::Item> {
        let mut archive_data: Option<Self::Item> = None;
        if self.get_next {
            if let Some((date, mut zipfile)) = self.archive.try_next() {
                match ArchiveData::new(&self.lid, &date, &mut zipfile) {
                    Ok(data) => {
                        archive_data.replace(data);
                    }
                    Err(error) => {
                        self.get_next = false;
                        log::error!("'{}' failed to create archive data on {}: {:?}", self.lid, date, error);
                    }
                }
            }
        }
        archive_data
    }
}

/// The iterator over archive content.
/// 
pub struct ArchiveContentIterator<R: Read + Seek> {
    /// The location alias.
    lid: String,
    /// The archive helper that return a history ZipFile
    archive: ArchiveIterator<R>,
    /// This allows the iterator to turn itself off.
    get_next: bool,
}
impl<R: Read + Seek> ArchiveContentIterator<R> {
    /// Create a new instance of the archive content iterator.
    ///
    /// # Arguments
    ///
    /// * `lid` is the location alias name.
    /// * `archive` is the zip archive that will be used by the iterator.
    /// * `dates` identifies what history dates will be used.
    ///
    pub fn new(lid: &str, archive: ZipArchive<R>, dates: Vec<NaiveDate>) -> Self {
        Self { lid: lid.into(), archive: ArchiveIterator::new(lid, archive, dates), get_next: true }
    }
}
/// Allow the collection archive content to be returned as an iterator.
///
impl<R: Read + Seek> Iterator for ArchiveContentIterator<R> {
    type Item = ArchiveContent;

    fn next(&mut self) -> Option<Self::Item> {
        let mut content: Option<Self::Item> = None;
        if self.get_next {
            if let Some((date, mut zipfile)) = self.archive.try_next() {
                match ArchiveData::new(&self.lid, &date, &mut zipfile) {
                    Ok(data) => {
                        let metadata = ArchiveMetadata::new(&date, &zipfile);
                        content.replace(ArchiveContent { metadata, data });
                    }
                    Err(error) => {
                        log::error!("'{}' failed to create archive data on {}: {:?}", self.lid, date, error);
                        self.get_next = false;
                    }
                }
            }
        }
        content
    }
}
