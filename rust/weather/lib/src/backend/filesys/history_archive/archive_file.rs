//! This is a complete rewrite of the old archive file implementation. When I moved
//! to the latest version of the zip crate the change to a generic ZipArchive pointed
//! out how far spread out the implementation had become.
//!
//! All zip file details are in this module hierarchy.
//!
mod reader;
use reader::ArchiveReader;

mod writer;
use writer::ArchiveWriter;
#[cfg(test)]
pub use writer::{BACKUP_EXT, UPDATE_EXT};

mod iterators;

use super::history;
use crate::{
    backend::filesys::WeatherFile,
    entities::{DateRange, History},
};
use chrono::NaiveDate;
use std::io::Read;
use zip::read::ZipFile;

/// The public API into the history archive.
#[derive(Debug)]
pub struct ArchiveFile {
    /// The unique identifier for a location.
    pub lid: String,
    /// The file that contains weather data.
    file: WeatherFile,
}
impl ArchiveFile {
    /// Create the manager for an existing weather data archive.
    ///
    /// An error will be returned if the archive does not exist or is not valid.
    ///
    /// # Arguments
    ///
    /// * `lid` is the location identifier.
    /// * `file` is the archive containing of weather data.
    ///
    pub fn open(lid: &str, file: WeatherFile) -> crate::Result<Self> {
        ArchiveReader::open(lid, &file)?;
        Ok(Self { lid: lid.to_string(), file })
    }

    /// Creates a new weather data archive and the manager for it
    ///
    /// An error will be returned if the archive exists or there are problems trying to create it.
    ///
    /// # Arguments
    ///
    /// * `lid` is the location identifier.
    /// * `file` is the container of weather data.
    pub fn create(lid: &str, file: WeatherFile) -> crate::Result<Self> {
        ArchiveReader::create(lid, &file)?;
        Ok(Self { lid: lid.to_string(), file })
    }

    /// Get the history dates from the weather archive.
    ///
    /// # Arguments
    ///
    /// * `filter` restricts history data to a range.
    /// * `sort` when true history dates will be returned in ascending order.
    ///
    pub fn history_dates(&self, selector: Option<&DateRange>, sort: bool) -> crate::Result<Vec<NaiveDate>> {
        let archive = ArchiveReader::open(&self.lid, &self.file)?;
        let mut dates = if let Some(date_range) = selector {
            archive.dates_from_date_range(date_range)?
        } else {
            archive.dates()?
        };
        if sort {
            dates.sort_unstable();
        }
        Ok(dates)
    }

    /// Get an iterator over archive metadata for a date range.
    ///
    /// # Arguments
    ///
    /// * `selector` provides the metadata history dates.
    ///
    pub fn metadata_iter(&self, selector: Option<&DateRange>) -> crate::Result<impl Iterator<Item = ArchiveMetadata>> {
        let archive_reader = ArchiveReader::open(&self.lid, &self.file)?;
        let mut dates = if let Some(date_range) = selector {
            archive_reader.dates_from_date_range(date_range)?
        } else {
            archive_reader.dates()?
        };
        dates.sort_unstable();
        let iter = archive_reader.metadata_by_date(dates)?;
        Ok(iter)
    }

    /// Get an iterator over archive metadata for a collection of dates.
    ///
    /// # Arguments
    ///
    /// * `selector` provides the metadata history dates.
    /// * `skip_not_found` will skip missing history dates otherwise if history is not found iteration will stop.
    ///
    pub fn metadata_by_date(
        &self,
        mut selector: Vec<NaiveDate>,
        skip_not_found: bool,
    ) -> crate::Result<impl Iterator<Item = ArchiveMetadata>> {
        let archive = ArchiveReader::open(&self.lid, &self.file)?;
        if skip_not_found {
            selector = selector.into_iter().filter(|date| archive.contains(date)).collect();
        }
        archive.metadata_by_date(selector)
    }

    /// Get an iterator over the file data for history dates.
    ///
    /// # Arguments
    ///
    /// * `selector` provides the metadata history dates.
    ///
    pub fn data_iter(&self, date_selector: &DateRange) -> crate::Result<Box<dyn Iterator<Item = ArchiveData>>> {
        let archive = ArchiveReader::open(&self.lid, &self.file)?;
        let mut dates = archive.dates_from_date_range(date_selector)?;
        dates.sort_unstable();
        let iterator = archive.data_by_date(dates)?;
        Ok(Box::new(iterator))
    }

    /// Get an iterator over the contents of an archive.
    ///
    /// # Arguments
    ///
    /// * `selector` provides the metadata history dates.
    ///
    pub fn content_iter(&self) -> crate::Result<Box<dyn Iterator<Item = ArchiveContent>>> {
        let archive = ArchiveReader::open(&self.lid, &self.file)?;
        let mut dates = archive.dates()?;
        dates.sort_unstable();
        let iterator = archive.content_by_date(dates)?;
        Ok(Box::new(iterator))
    }

    /// Add history data to the archive.
    ///
    /// #Arguments
    ///
    /// * `data` contains the archive file contents.
    ///
    pub fn add_data(&self, data: Vec<ArchiveData>) -> crate::Result<()> {
        ArchiveWriter::new(&self.lid, &self.file).add_data(data)
    }

    /// Get the size of the file.
    ///
    pub fn size(&self) -> u64 {
        self.file.size()
    }
}

/// A bean providing stats about a weather history file in the archive.
#[derive(Debug)]
pub struct ArchiveMetadata {
    /// The date associated with the history file in the archive.
    pub date: NaiveDate,
    /// The size of the file in the archive.
    pub compressed_size: u64,
    /// The actual size of the file.
    pub size: u64,
}
impl ArchiveMetadata {
    /// Create a new instance of the metadata.
    ///
    /// # Arguments
    ///
    /// * `date` is the zip file history date.
    /// * `zipfile` is the archive zip file.
    ///
    pub(self) fn new<'r, R: Read>(date: &NaiveDate, zipfile: &'r ZipFile<R>) -> Self {
        Self { date: date.clone(), compressed_size: zipfile.compressed_size(), size: zipfile.size() }
    }
}
impl std::fmt::Display for ArchiveMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A bean providing raw archive file data.
///
#[derive(Debug)]
pub struct ArchiveData {
    pub lid: String,
    pub date: NaiveDate,
    pub data: Vec<u8>,
}
impl ArchiveData {
    /// Create a new instance of the archive data.
    ///
    /// # Arguments
    ///
    /// * `lid` is the location alias.
    /// * `date` is the history date.
    /// * `zipfile` is the archive zip file.
    ///
    pub(self) fn new<'r, R: Read>(lid: &str, date: &NaiveDate, zipfile: &'r mut ZipFile<R>) -> crate::Result<Self> {
        let size = zipfile.size() as usize;
        let mut data: Vec<u8> = Vec::with_capacity(size);
        if let Err(error) = zipfile.read_to_end(&mut data) {
            Err(crate::Error::from(format!("'{}' history file error: {:?}", lid, error)))
        } else {
            Ok(Self { lid: lid.into(), date: date.clone(), data })
        }
    }
}
/// Convert the archive file data into a History instance.
///
impl TryFrom<ArchiveData> for History {
    type Error = crate::Error;
    fn try_from(archive_data: ArchiveData) -> Result<Self, Self::Error> {
        history::from_bytes(&archive_data.lid, &archive_data.data)
    }
}

/// A bean providing raw archive file content.
///
pub struct ArchiveContent {
    /// The archive file metadata.
    pub metadata: ArchiveMetadata,
    /// The archive file data.
    pub data: ArchiveData,
}
/// Convert the archive content into metadata and History.
///
impl TryFrom<ArchiveContent> for (ArchiveMetadata, History) {
    type Error = crate::Error;
    fn try_from(content: ArchiveContent) -> Result<Self, Self::Error> {
        Ok((content.metadata, content.data.try_into()?))
    }
}

mod archive {
    //! Consolidate the history filename utilities to this module.

    use super::WeatherFile;
    use chrono::NaiveDate;
    use std::fs::File;
    use std::io::BufReader;
    use zip::ZipArchive;

    /// Creates the ZipArchive that will read data out of the archive file.
    ///
    /// # Arguments
    ///
    /// * `file` is the weather history file the ZipArchive will usee.
    ///
    pub fn open(file: &WeatherFile) -> crate::Result<ZipArchive<BufReader<File>>> {
        match ZipArchive::new(BufReader::new(file.reader()?)) {
            Ok(archive) => Ok(archive),
            Err(error) => Err(crate::Error::from(format!("Error opening archive: {:?}", error))),
        }
    }

    /// Build the internal archive filename to the provided date.
    ///
    /// # Arguments
    ///
    /// * `lid` is the location id.
    /// * `date` is the history date that will be embedded into the filename.
    pub fn date_to_filename(lid: &str, date: &NaiveDate) -> String {
        format!("{}/{}-{}.json", lid, lid, date.format("%Y%m%d"))
    }

    /// Extracts the date from internal archive filename.
    ///
    /// An error is returned if the filename is not a valid history name.
    ///
    /// # Arguments
    ///
    /// * `history_name` is a weather archive filename containing the embedded date.
    pub fn filename_to_date(filename: &str) -> crate::Result<NaiveDate> {
        let ymd_offset = "yyyymmdd.json".len();
        if ymd_offset > filename.len() {
            Err(crate::Error::from(format!("malformed history name: {}.", filename)))
        } else {
            let ymd_index = filename.len() - ymd_offset;
            let ymd: &str = &filename[ymd_index..ymd_index + 8];
            if !ymd.chars().all(char::is_numeric) {
                Err(crate::Error::from(format!("history date not found in '{}'.", filename)))
            } else {
                let year = ymd[..4].parse().unwrap();
                let month = ymd[4..6].parse().unwrap();
                let day = ymd[6..].parse().unwrap();
                match NaiveDate::from_ymd_opt(year, month, day) {
                    Some(date) => Ok(date),
                    None => Err(crate::Error::from(format!("illegal date from history name '{}'.", filename))),
                }
            }
        }
    }
}
