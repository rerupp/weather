//! The current implementation of administration for the file system.

use crate::{
    admin_prelude::{FilesysDetails, FilesysProblems, LocationDetails},
    backend::filesys::{fs_lib, history_archive::HistoryArchive, Locations, WeatherDir},
};
use std::{fmt::Formatter, rc::Rc};

pub(in crate::backend) fn create_fs_admin(weather_dir: Rc<WeatherDir>) -> FsAdmin {
    FsAdmin { weather_dir }
}

macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error(format!("FsAdmin {}.", format!($($arg)*))))
    };
}

pub(crate) struct FsAdmin {
    weather_dir: Rc<WeatherDir>,
}
impl std::fmt::Debug for FsAdmin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FsAdmin({})", self.weather_dir)
    }
}
impl FsAdmin {
    /// Initialize the weather data files.
    ///
    pub fn init(&self) -> crate::Result<()> {
        if Locations::exists(&self.weather_dir) {
            log::warn!("The weather data directory has already been initialized.");
        } else if let Err(error) = Locations::open(&self.weather_dir) {
            err!("{error}")?;
        }
        Ok(())
    }

    /// Check the weather data files looking for inconsistencies.
    ///
    /// # Arguments
    ///
    /// * `repair` when true will try to fix issues that were found.
    ///
    pub fn check(&self, repair: bool) -> Option<FilesysProblems> {
        match internal::DataScanner::new(self.weather_dir.clone(), repair) {
            Err(document_problem) => Some(FilesysProblems::from(document_problem)),
            Ok(scanner) => scanner.run(),
        }
    }

    /// Get the weather data location file details.
    ///
    pub fn details(&self) -> crate::Result<FilesysDetails> {
        let mut location_details = vec![];
        let mut archives_size: u64 = 0;
        for location in fs_lib::get_locations(&self.weather_dir, None)? {
            let file = self.weather_dir.archive(&location.alias);
            archives_size += file.size();
            let mut histories: usize = 0;
            let compressed_size: usize = HistoryArchive::open(&location.alias, file)?
                .metadata()?
                .map(|metadata| {
                    histories += 1;
                    metadata.compressed_size as usize
                })
                .sum();
            location_details.push(LocationDetails { alias: location.alias.clone(), size: compressed_size, histories })
        }
        Ok(FilesysDetails { size: archives_size as usize, location_details })
    }

    /// Copy the contents of a location weather history archive to another alias name.
    ///
    /// # Arguments
    ///
    /// * `source` is the location alias name of the weather history that will be copied.
    /// * `destination` is the location alias name of the copied weather history.
    ///
    pub fn copy_archive(&self, source: &str, destination: &str) -> crate::Result<()> {
        crate::log_elapsed_time!(&format!("copy_archive({source}, {destination}):"));

        // at this point in the stack the archives should always be there but JIC...
        let source_file = self.weather_dir.archive(source);
        if !source_file.exists() {
            err!("The source archive ({source_file}) does not exist.")?;
        }
        let source_archive = HistoryArchive::open(source, source_file)?;

        let destination_file = self.weather_dir.archive(destination);
        if !destination_file.exists() {
            err!("The destination archive ({destination_file}) does not exists.")?;
        }
        let destination_archive = HistoryArchive::open(destination, destination_file)?;
        if !destination_archive.is_empty() {
            err!("The destination history archive is not empty.")?;
        }

        let result = source_archive.copy(&destination_archive);
        if result.is_err() {
            // remove the destination archive
            if let Err(error) = self.weather_dir.archive(destination).remove() {
                log::error!("Error removing destination archive '{destination}': {error:?}");
            }
        }
        result
    }
}

mod internal {
    //! Utilities internal to the filesystem administration API.
    //!
    use super::*;
    use crate::{
        admin_prelude::{FilesysDocumentProblem, FilesysLocationProblem, FilesysProblems},
        backend::filesys::WeatherFile,
        prelude::Location,
    };
    use std::{ffi::OsStr, fs, io};

    /// The weather data file scanner.
    ///
    pub struct DataScanner {
        /// The weather data directory.
        weather_dir: Rc<WeatherDir>,
        /// When `true ` attempt to fix location weather history archive problems.
        repair: bool,
        /// The current collection of weather data locations.
        locations: Vec<Location>,
    }
    impl DataScanner {
        /// Create the data scanner and return a document problem if there are errors.
        ///
        /// # Arguments
        ///
        /// * `weather_dir` is the weather data directory.
        /// * `repair` controls if archive problems should be repaired of not.
        ///
        pub fn new(weather_dir: Rc<WeatherDir>, repair: bool) -> Result<Self, FilesysDocumentProblem> {
            // don't use the fs_lib function so you can differentiate between an open problem and a read problem
            match Locations::open(&weather_dir) {
                Err(error) => Err(FilesysDocumentProblem::open_error(error)),
                Ok(locations) => match locations.get(None) {
                    Err(error) => Err(FilesysDocumentProblem::read_error(error)),
                    Ok(iter) => Ok(Self { weather_dir, repair, locations: iter.collect() }),
                },
            }
        }

        /// Run the weather data scanner.
        ///
        pub fn run(&self) -> Option<FilesysProblems> {
            let mut filesys_problems_opt = None;
            match self.scan_location_archives() {
                Some(location_problems) => {
                    let mut filesys_problems = FilesysProblems::from(location_problems);
                    filesys_problems.detached_archives = self.scan_detached_archives();
                    filesys_problems_opt.replace(filesys_problems);
                }
                None => {
                    if let Some(detached_archives) = self.scan_detached_archives() {
                        let mut filesys_problems = FilesysProblems::default();
                        filesys_problems.detached_archives.replace(detached_archives);
                        filesys_problems_opt.replace(filesys_problems);
                    }
                }
            }
            filesys_problems_opt
        }

        /// Scan location weather data archives looking for problems.
        ///
        fn scan_location_archives(&self) -> Option<Vec<FilesysLocationProblem>> {
            let mut problems = vec![];
            for location in &self.locations {
                let archive = self.weather_dir.archive(&location.alias);
                let location_problem = match archive.exists() {
                    true => self.open_archive(location),
                    false => Some(self.missing_archive(location)),
                };
                if let Some(problem) = location_problem {
                    problems.push(problem);
                }
            }
            match problems.len() > 0 {
                true => Some(problems),
                false => None,
            }
        }

        /// Describe the missing location weather history archive problem and optionally try to fix it.
        ///
        /// # Arguments
        ///
        /// * `location` is the location with the missing archive problem.
        ///
        fn missing_archive(&self, location: &Location) -> FilesysLocationProblem {
            let mut problem = FilesysLocationProblem::from(location);
            problem.missing_archive = true;
            if self.repair {
                let archive = self.weather_dir.archive(&location.alias);
                match HistoryArchive::create(&location.alias, archive) {
                    Ok(_) => problem.repaired = true,
                    Err(error) => {
                        problem.create_error.replace(error);
                    }
                }
            }
            problem
        }

        /// Verify the location weather history archive is okay and optionally try to fix any errors.
        ///
        /// # Arguments
        ///
        /// * `location` identifies the location weather history archive that will be opened.
        ///
        fn open_archive(&self, location: &Location) -> Option<FilesysLocationProblem> {
            let mut problem_opt = None;
            let archive = self.weather_dir.archive(&location.alias);
            if let Err(error) = HistoryArchive::open(&location.alias, archive) {
                let mut problem = FilesysLocationProblem::from(location);
                problem.open_error.replace(error);
                if self.repair {
                    let archive = self.weather_dir.archive(&location.alias);
                    match archive.remove() {
                        Err(error) => {
                            problem.create_error.replace(error);
                        }
                        Ok(_) => match HistoryArchive::create(&location.alias, archive) {
                            Ok(_) => problem.repaired = true,
                            Err(error) => {
                                problem.create_error.replace(error);
                            }
                        },
                    }
                }
                problem_opt.replace(problem);
            }
            problem_opt
        }

        /// Scan the weather data folder looking for archives that do not have associated locations.
        ///
        fn scan_detached_archives(&self) -> Option<Vec<String>> {
            match self.get_archive_files() {
                Err(error) => {
                    log::error!("Error getting the weather directory archive collection: {error}.");
                    None
                }
                Ok(archives_files) => {
                    let location_archives = self
                        .locations
                        .iter()
                        .map(|location| self.weather_dir.archive(&location.alias))
                        .collect::<Vec<_>>();
                    let mut detached_archives = vec![];
                    for archive in archives_files {
                        if !location_archives.iter().any(|file| file.filename == archive.filename) {
                            detached_archives.push(archive.filename);
                        }
                    }
                    match detached_archives.len() > 0 {
                        true => Some(detached_archives),
                        false => None,
                    }
                }
            }
        }

        /// Get the list of files that appear to be weather history archives.
        ///
        fn get_archive_files(&self) -> io::Result<Vec<WeatherFile>> {
            let archive_extension = OsStr::new(WeatherDir::ARCHIVE_EXTENSION);
            let mut archives = vec![];
            for entry in fs::read_dir(self.weather_dir.path())? {
                let path = entry?.path();
                if path.extension() == Some(archive_extension) {
                    archives.push(self.weather_dir.archive(path.file_name().unwrap().to_str().unwrap()))
                }
            }
            Ok(archives)
        }
    }
}
