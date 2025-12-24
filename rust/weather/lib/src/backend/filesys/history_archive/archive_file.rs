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
        crate::log_elapsed_time!("ArchiveFile::add_data():");
        ArchiveWriter::new(&self.lid, &self.file).add_data(data)
    }

    pub fn copy_filter(&self, destination: &ArchiveFile, date_filter: Option<Vec<NaiveDate>>) -> crate::Result<()> {
        macro_rules! err {
            ($($arg:tt)*) => {
                Err(crate::Error::from(format!("ArchiveFile({}).copy_filter({}): {}", self.lid, destination.lid, format!($($arg)*))))
            };
        }
        let mut reader = archive::reader(&self.file)?;
        let mut filenames = reader.file_names().map(|n| n.to_string()).collect::<Vec<_>>();
        filenames.sort_unstable();
        let mut writer = archive::writer(&destination.file)?;
        for source_filename in &filenames {
            match reader.by_name(source_filename) {
                Err(error) => err!("error getting history file {source_filename}: {error}")?,
                Ok(zip_file) => {
                    let date = archive::filename_to_date(source_filename)?;
                    if let Some(date_filter) = &date_filter {
                        if !date_filter.contains(&date) {
                            continue;
                        }
                    }
                    let destination_filename = archive::date_to_filename(&destination.lid, &date);
                    if let Err(error) = writer.raw_copy_file_rename(zip_file, &destination_filename) {
                        err!("error copying archive file {destination_filename}: {error}")?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Check to see if the archive does not contain any files.
    ///
    pub fn is_empty(&self) -> bool {
        match archive::reader(&self.file) {
            Ok(reader) => reader.is_empty(),
            Err(error) => {
                log::error!("ArchiveFile({}): {}", self.lid, error);
                false
            }
        }
    }

    /// Get the archive file count.
    ///
    pub fn history_count(&self) -> usize {
        match archive::reader(&self.file) {
            Ok(reader) => reader.len(),
            Err(error) => {
                log::error!("ArchiveFile({}): {}", self.lid, error);
                0
            }
        }
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
    use zip::{ZipArchive, ZipWriter};

    /// Creates the [ZipArchive] that will read data out of the archive file.
    ///
    /// # Arguments
    ///
    /// * `archive` is the weather history file the [ZipArchive] reads.
    ///
    pub fn reader(archive: &WeatherFile) -> crate::Result<ZipArchive<BufReader<File>>> {
        match ZipArchive::new(BufReader::new(archive.reader()?)) {
            Ok(archive) => Ok(archive),
            Err(error) => Err(crate::Error::from(format!("Error opening archive: {:?}", error))),
        }
    }

    /// Creates the actual [ZipWriter] that will update the archive.
    ///
    /// # Arguments
    ///
    /// * `archive` is the weather history file the [ZipWriter] will update.
    ///
    pub fn writer(archive: &WeatherFile) -> crate::Result<ZipWriter<File>> {
        match File::options().read(true).write(true).create(true).open(archive.path()) {
            Ok(file) => match ZipWriter::new_append(file) {
                Ok(zip_writer) => Ok(zip_writer),
                Err(zip_error) => Err(crate::Error(format!("failed to open file: {zip_error}"))),
            },
            Err(file_error) => Err(crate::Error(format!("failed to create file: {}.", file_error))),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{testlib, WeatherDir};
    use std::path::PathBuf;

    macro_rules! date {
        ($y:expr, $m:expr, $d:expr) => {
            NaiveDate::from_ymd_opt($y, $m, $d).unwrap()
        };
    }

    #[test]
    fn copy_filter() {
        // use the database test resources
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("db"));
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();

        // make sure the copy without filter is working
        let source = ArchiveFile::open("north", weather_dir.archive("north")).unwrap();
        let copy = ArchiveFile::create("copy", weather_dir.archive("copy")).unwrap();
        source.copy_filter(&copy, None).unwrap();
        let source_dates = source.history_dates(None, true).unwrap();
        let copy_dates = copy.history_dates(None, true).unwrap();
        for (lhs, rhs) in source_dates.iter().zip(copy_dates.iter()) {
            assert_eq!(lhs, rhs);
        }

        let filtered = ArchiveFile::create("filtered", weather_dir.archive("filtered")).unwrap();
        let mut date_filter = DateRange::new(date!(2015, 4, 1), date!(2015, 4, 14)).iter().collect::<Vec<_>>();
        DateRange::new(date!(2016, 10, 10), date!(2016, 10, 17)).iter().for_each(|date| date_filter.push(date));
        DateRange::new(date!(2017, 7, 14), date!(2017, 7, 20)).iter().for_each(|date| date_filter.push(date));
        source.copy_filter(&filtered, Some(date_filter.clone())).unwrap();
        let filtered_dates = filtered.history_dates(None, true).unwrap();
        for (lhs, rhs) in date_filter.iter().zip(filtered_dates.iter()) {
            assert_eq!(lhs, rhs);
        }
    }

    // #[test]
    #[allow(unused)]
    fn db_test_archives() {
        let weather_dir = WeatherDir::try_from("temp").unwrap();

        // north and between share the same date ranges
        let mut date_filter = DateRange::new(date!(2015, 4, 1), date!(2015, 4, 14)).iter().collect::<Vec<_>>();
        DateRange::new(date!(2016, 10, 10), date!(2016, 10, 17)).iter().for_each(|date| date_filter.push(date));
        DateRange::new(date!(2017, 7, 14), date!(2017, 7, 20)).iter().for_each(|date| date_filter.push(date));
        let tigard = ArchiveFile::open("tigard", weather_dir.archive("tigard")).unwrap();
        let north = ArchiveFile::create("north", weather_dir.archive("north")).unwrap();
        tigard.copy_filter(&north, Some(date_filter.clone())).unwrap();
        let carson_city_nv = ArchiveFile::open("carson_city_nv", weather_dir.archive("carson_city_nv")).unwrap();
        let between = ArchiveFile::create("between", weather_dir.archive("between")).unwrap();
        carson_city_nv.copy_filter(&between, Some(date_filter)).unwrap();

        // of course south has to be different
        let mut date_filter = DateRange::new(date!(2015, 4, 1), date!(2015, 4, 14)).iter().collect::<Vec<_>>();
        DateRange::new(date!(2016, 10, 10), date!(2016, 10, 17)).iter().for_each(|date| date_filter.push(date));
        DateRange::new(date!(2018, 1, 1), date!(2018, 1, 7)).iter().for_each(|date| date_filter.push(date));
        let foothills = ArchiveFile::open("foothills", weather_dir.archive("foothills")).unwrap();
        let south = ArchiveFile::create("south", weather_dir.archive("south")).unwrap();
        foothills.copy_filter(&south, Some(date_filter)).unwrap();
    }
}
