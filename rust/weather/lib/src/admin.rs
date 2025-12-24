//! The weather data administration API and data beans.

use crate::{
    admin_prelude::{Components, UsCityDetails},
    backend::{
        self,
        admin::{DbAdmin, FsAdmin},
        WeatherDir,
    },
    prelude::{Configuration, Location, LocationFilter, WeatherData},
};
use std::rc::Rc;

macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error(format!("WeatherAdmin {}.", format!($($arg)*))))
    };
}

/// The weather data administration `API`.
pub struct WeatherAdmin {
    /// The database administration commands.
    db_admin: Option<Box<dyn DbAdmin>>,
    /// The filesystem administration commands.
    fs_admin: FsAdmin,
    /// The configuration properties.
    configuration: Configuration,
    /// Some administration commands need the weather data api so make sure it
    /// uses the same configuration.
    pub weather_data: WeatherData,
}
impl std::fmt::Debug for WeatherAdmin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WeatherAdmin({})", self.configuration.weather_data.directory)
    }
}
impl TryFrom<Configuration> for WeatherAdmin {
    type Error = crate::Error;
    fn try_from(configuration: Configuration) -> Result<Self, Self::Error> {
        let weather_dir = Rc::new(WeatherDir::try_from(&configuration)?);
        let db_admin = match configuration.weather_data.fs_only {
            true => None,
            false => Some(backend::admin::create_db_admin(weather_dir.clone())),
        };
        let fs_admin = backend::admin::create_fs_admin(weather_dir);
        let weather_data = WeatherData::try_from(configuration.clone())?;
        Ok(Self { db_admin, fs_admin, weather_data, configuration })
    }
}
impl WeatherAdmin {
    /// Initialize the weather data directory.
    ///
    /// # Arguments
    ///
    /// * `update` when `true` will run initialization even if the weather data directory appears initialized.
    /// * `load` when `true` will load weather data into the database.
    /// * `threads` controls the number of history readers used to load weather data into the database.
    ///
    pub fn init(&self, update: bool, load: bool, threads: usize) -> crate::Result<()> {
        crate::log_elapsed_time!(&format!("init({load}, {threads}):"));
        self.fs_admin.init()?;
        if let Some(db_admin) = &self.db_admin {
            if db_admin.history_init(update)? {
                if load {
                    // unless something is really AFU this will always return details
                    let db_details = db_admin.history_details()?.unwrap();
                    if db_details.location_details.len() > 0 {
                        log::warn!("The database already contains history data.")
                    } else {
                        db_admin.history_load(threads)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Check the consistency of the weather data directory.
    ///
    /// # Arguments
    ///
    /// * `repair` when true will try to fix problems that were found.
    ///
    pub fn check(&self, repair: bool) -> (Option<entities::FilesysProblems>, Option<entities::DbProblems>) {
        let filesys_problems = self.fs_admin.check(repair);
        let db_problems = self.db_admin.as_ref().map_or(None, |db_admin| db_admin.history_check(repair));
        (filesys_problems, db_problems)
    }

    /// Deletes the weather database schema and optionally deletes the database.
    ///
    /// # Arguments
    ///
    /// * `delete` when `true` will delete the database file.
    ///
    pub fn drop(&self, delete: bool) -> crate::Result<()> {
        crate::log_elapsed_time!(&format!("delete({delete}):"));
        if let Some(db_admin) = &self.db_admin {
            db_admin.history_drop(delete)?;
        }
        Ok(())
    }

    /// Copy the weather history of a location to a new location. The copy location
    /// identifier is the only change made to the original locations properties.
    ///
    /// # Arguments
    ///
    /// * `source_alias` is the source location identifier.
    /// * `destination` is the new location that will get a copy of the weather history.
    ///
    pub fn copy_location(&self, source_alias: &str, destination: Location) -> crate::Result<()> {
        crate::log_elapsed_time!("WeatherAdmin copy location:");
        let filter = |name: &str| LocationFilter::name(name);
        // make sure the source exists
        let location = match self.weather_data.backend.get_locations(Some(vec![filter(source_alias)])) {
            Err(error) => err!("{error}")?,
            Ok(mut locations) => match locations.len() {
                0 => err!("the source location ({source_alias}) was not found.")?,
                1 => Ok(locations.remove(0)),
                _ => err!("multiple locations were found for '{source_alias}'"),
            },
        }?;

        // create the destination location
        let destination_alias = destination.alias.clone();
        if let Err(error) = self.weather_data.backend.add_location(destination) {
            err!("copy failed to create the destination location: {error}")?;
        }

        // copy the source archive and make sure to use the location alias, the source alias might have wildcards
        if let Err(copy_error) = self.fs_admin.copy_archive(&location.alias, &destination_alias) {
            // clean up what has been done
            if let Err(error) = self.weather_data.backend.delete_location(filter(&destination_alias)) {
                log::error!("failed to cleanup the destination location '{destination_alias}': {error}.")
            }
            err!("failed to copy the source archive: {copy_error}")?;
        }

        if let Some(db_admin) = &self.db_admin {
            let destination_filter = LocationFilter::name(&destination_alias);
            if let Err(reload_error) = db_admin.history_reload(vec![destination_filter]) {
                // don't fall over if the archive cannot be deleted
                if let Err(error) = self.weather_data.backend.delete_location(filter(&destination_alias)) {
                    log::error!("failed to cleanup the destination location '{destination_alias}': {error}.")
                }
                err!("failed to load the location histories: {reload_error}")?;
            }
        }
        Ok(())
    }

    /// Provides information about the weather data archives and database.
    ///
    pub fn components(&self) -> crate::Result<Components> {
        crate::log_elapsed_time!("components():");
        let fs_details = self.fs_admin.details()?;
        let db_details = match &self.db_admin {
            None => None,
            Some(db_admin) => db_admin.history_details()?,
        };
        Ok(Components { db_details, fs_details })
    }

    /// Reload history for locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations that will be reloaded.
    ///
    pub fn reload(&self, filters: Vec<LocationFilter>) -> crate::Result<usize> {
        crate::log_elapsed_time!("reload():");
        let count = match &self.db_admin {
            None => 0,
            Some(db_admin) => db_admin.history_reload(filters)?,
        };
        Ok(count)
    }

    /// Initialize the US Cities database.
    ///
    pub fn uscities_init(&self) -> crate::Result<()> {
        crate::log_elapsed_time!("uscities_init():");
        if let Some(db_admin) = &self.db_admin {
            db_admin.us_cities_init()?;
        }
        Ok(())
    }

    /// Load the US Cities database.
    ///
    pub fn uscities_load(&self) -> crate::Result<()> {
        let uscities_filename = self.configuration.us_cities.filename.as_str();
        crate::log_elapsed_time!(&format!("uscities_load({uscities_filename}):"));
        if let Some(db_admin) = &self.db_admin {
            db_admin.us_cities_load(uscities_filename)?;
        }
        Ok(())
    }

    /// Delete the US Cities database.
    ///
    pub fn uscities_delete(&self) -> crate::Result<()> {
        crate::log_elapsed_time!("uscities_delete():");
        if let Some(db_admin) = &self.db_admin {
            db_admin.us_cities_delete()?;
        }
        Ok(())
    }

    /// Show information about the US Cities database.
    ///
    pub fn uscities_info(&self) -> crate::Result<Option<UsCityDetails>> {
        crate::log_elapsed_time!("uscities_info():");
        match &self.db_admin {
            None => Ok(None),
            Some(db_admin) => Ok(Some(db_admin.us_cities_details()?)),
        }
    }
}

pub mod entities {
    //! Entities specific to the administration API.
    //!

    use super::Location;
    use crate::prelude::HistorySummaries;

    /// The administration backend component information.
    #[derive(Debug)]
    pub struct Components {
        /// The database information.
        pub db_details: Option<DbDetails>,
        /// The archive information.
        pub fs_details: FilesysDetails,
    }

    /// The database information.
    #[derive(Debug)]
    pub struct DbDetails {
        /// The size of the database.
        pub size: usize,
        /// The location weather history information.
        pub location_details: Vec<LocationDetails>,
    }

    #[derive(Debug, Default)]
    pub struct DbProblems {
        pub db_error: Option<crate::Error>,
        pub location_problems: Option<DbLocationProblems>,
        pub history_problems: Option<DbHistoryProblems>,
    }
    impl From<crate::Error> for DbProblems {
        fn from(error: crate::Error) -> Self {
            Self { db_error: Some(error), location_problems: None, history_problems: None }
        }
    }

    #[derive(Debug, Default)]
    pub struct DbLocationProblems {
        pub missing_locations: Option<Vec<Location>>,
        pub detached_locations: Option<Vec<Location>>,
    }

    #[derive(Debug, Default)]
    pub struct DbHistoryProblems {
        /// The database and filesystem have different history counts.
        pub history_problems: Option<Vec<DbHistoryProblemDetails>>,
        /// The database location does not have a corresponding filesystem location.
        pub detached_store: Option<Vec<HistorySummaries>>,
    }

    #[derive(Debug)]
    pub struct DbHistoryProblemDetails {
        pub location: Location,
        pub db_histories: usize,
        pub fs_histories: usize,
    }

    #[derive(Debug, Default)]
    pub struct FilesysDocumentProblem {
        pub open_error: Option<crate::Error>,
        pub read_error: Option<crate::Error>,
    }
    impl FilesysDocumentProblem {
        pub fn open_error(error: crate::Error) -> Self {
            Self { open_error: Some(error), read_error: None }
        }
        pub fn read_error(error: crate::Error) -> Self {
            Self { open_error: None, read_error: Some(error) }
        }
    }
    /// The details about a problem found with a locations weather history data.
    ///
    #[derive(Debug)]
    pub struct FilesysLocationProblem {
        /// The location associated with a problem.
        pub location: Location,
        /// `true` if the problem was repaired.
        pub repaired: bool,
        /// `true` if the archive was missing.
        pub missing_archive: bool,
        /// The result of opening the weather history archive.
        pub open_error: Option<crate::Error>,
        /// The result of trying to create the weather history archive.
        pub create_error: Option<crate::Error>,
    }
    impl From<Location> for FilesysLocationProblem {
        fn from(location: Location) -> Self {
            Self { location, repaired: false, missing_archive: false, create_error: None, open_error: None }
        }
    }
    impl From<&Location> for FilesysLocationProblem {
        fn from(location: &Location) -> Self {
            Self::from(location.clone())
        }
    }

    /// The results of checking the weather history data files.
    ///
    #[derive(Debug, Default)]
    pub struct FilesysProblems {
        /// The error associated with the locations document.
        pub document_problem: Option<FilesysDocumentProblem>,
        /// The collection of locations with problems.
        pub location_problems: Option<Vec<FilesysLocationProblem>>,
        /// A list of weather history archive files that do not have an associated location.
        pub detached_archives: Option<Vec<String>>,
    }
    impl From<FilesysDocumentProblem> for FilesysProblems {
        fn from(problem: FilesysDocumentProblem) -> Self {
            Self { document_problem: Some(problem), location_problems: None, detached_archives: None }
        }
    }
    impl From<Vec<FilesysLocationProblem>> for FilesysProblems {
        fn from(problems: Vec<FilesysLocationProblem>) -> Self {
            Self { document_problem: None, location_problems: Some(problems), detached_archives: None }
        }
    }

    /// Information about the weather history archives.
    #[derive(Debug, Default)]
    pub struct FilesysDetails {
        /// The total size of weather history archives.
        pub size: usize,
        /// The location information
        pub location_details: Vec<LocationDetails>,
    }

    /// Weather history metadata for a [location](Location).
    #[derive(Debug)]
    pub struct LocationDetails {
        /// The location alias name.
        pub alias: String,
        /// The number of bytes being used to hold weather history information.
        pub size: usize,
        /// The count of weather histories the [location](Location) has available.
        pub histories: usize,
    }

    #[derive(Debug)]
    pub struct UsCityDetails {
        pub db_size: usize,
        pub state_info: Vec<(String, usize)>,
    }
}
