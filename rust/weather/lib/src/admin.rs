//! The weather data administration API and data beans.

use crate::{
    admin_prelude::{CitiesDetails, Components},
    backend::{
        self,
        admin::{DbAdmin, FsAdmin},
        WeatherDir,
    },
    prelude::{Configuration, Location, LocationFilter, WeatherData},
};
use std::rc::Rc;

/// Create an error from the locations specific error message.
///
/// # Params
///
/// * `args` will be passed to `format!` to create the error message.
///
macro_rules! err {
    ($($args:tt)*) => {
        Err(crate::Error(format!("WeatherAdmin {}.", format!($($args)*))))
    };
}

/// This is boilerplate code used when the Cities db is not being used.
///
macro_rules! cities_not_available {
    () => {
        log::info!("Cities is only available when a db is used.")
    };
}

/// The weather data administration `API`.
///
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
        let location_opt = self.weather_data.get_location(LocationFilter::alias(source_alias))?;
        if location_opt.is_none() {
            log::warn!("WeatherAdmin copy: the source location does not exist.");
            return Ok(());
        }
        let location = location_opt.unwrap();

        // create the destination location
        let destination_alias = destination.alias.clone();
        if let Err(error) = self.weather_data.backend.add_location(destination) {
            err!("copy failed to create the destination location: {error}")?;
        }

        // copy the source archive and make sure to use the location alias, the source alias might have wildcards
        if let Err(copy_error) = self.fs_admin.copy_archive(&location.alias, &destination_alias) {
            // clean up what has been done
            let destination_filter = LocationFilter::alias(&destination_alias);
            if let Err(error) = self.weather_data.backend.delete_location(destination_filter) {
                log::error!("failed to cleanup the destination location '{destination_alias}': {error}.")
            }
            err!("failed to copy the source archive: {copy_error}")?;
        }

        // add the new locations weather history if using a database
        if let Some(db_admin) = &self.db_admin {
            let destination_filter = LocationFilter::alias(&destination_alias);
            if let Err(reload_error) = db_admin.history_reload(vec![destination_filter]) {
                // don't fall over if the archive cannot be deleted
                let destination_filter = LocationFilter::alias(&destination_alias);
                if let Err(error) = self.weather_data.backend.delete_location(destination_filter) {
                    log::error!("failed to cleanup the destination location '{destination_alias}': {error}.")
                }
                err!("failed to load the location histories: {reload_error}")?;
            }
        }
        Ok(())
    }

    /// Compress a locations weather history archive and return the space that was recovered.
    ///
    /// # Arguments
    ///
    /// * `location` is used to select the weather history archive.
    ///
    pub fn compress_archive(&self, location: &Location) -> crate::Result<u64> {
        self.fs_admin.compress_archive(&location.alias)
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

    /// Initialize the Cities database.
    ///
    pub fn cities_init(&self) -> crate::Result<()> {
        match &self.db_admin {
            None => cities_not_available!(),
            Some(db_admin) => {
                crate::log_elapsed_time!("cities_init():");
                db_admin.cities_init()?;
            }
        }
        Ok(())
    }

    /// Load a Simple Maps country CSV database into the Cities database.
    ///
    /// # Arguments
    ///
    /// * `csv_database` is the path to the CSV database file.
    /// * `reload` will remove existing country data before loading what is mined from the file.
    ///
    pub fn cities_load(&self, csv_database: String, reload: bool) -> crate::Result<()> {
        match &self.db_admin {
            None => cities_not_available!(),
            Some(db_admin) => {
                crate::log_elapsed_time!(&format!("uscities_load({csv_database}):"));
                db_admin.cities_load(&csv_database, reload)?;
            }
        }
        Ok(())
    }

    /// Delete the Cities database.
    ///
    pub fn cities_delete(&self) -> crate::Result<()> {
        match &self.db_admin {
            None => cities_not_available!(),
            Some(db_admin) => {
                crate::log_elapsed_time!("cities_delete():");
                db_admin.cities_delete()?
            }
        }
        Ok(())
    }

    /// Show information about the Cities database.
    ///
    pub fn cities_details(&self) -> crate::Result<Option<CitiesDetails>> {
        match &self.db_admin {
            Some(db_admin) => db_admin.cities_details(),
            None => {
                cities_not_available!();
                Ok(None)
            }
        }
    }
}

pub mod entities {
    //! Entities specific to the administration API.
    //!
    use super::Location;
    use crate::prelude::HistorySummaries;

    /// The administration backend component information.
    ///
    #[derive(Debug)]
    pub struct Components {
        /// The database information.
        pub db_details: Option<DbDetails>,
        /// The archive information.
        pub fs_details: FilesysDetails,
    }

    /// The database information.
    ///
    #[derive(Debug)]
    pub struct DbDetails {
        /// The size of the database.
        pub size: usize,
        /// The location weather history information.
        pub location_details: Vec<LocationDetails>,
    }

    /// Problems that were found in the database.
    ///
    #[derive(Debug, Default)]
    pub struct DbProblems {
        /// There was some problem with the database.
        pub db_error: Option<crate::Error>,
        /// There were problems with the loaded locations.
        pub location_problems: Option<DbLocationProblems>,
        /// There were problems with the loaded weather history data.
        pub history_problems: Option<DbHistoryProblems>,
    }
    impl From<crate::Error> for DbProblems {
        fn from(error: crate::Error) -> Self {
            Self { db_error: Some(error), location_problems: None, history_problems: None }
        }
    }

    /// Problems that were found with the database locations.
    ///
    #[derive(Debug, Default)]
    pub struct DbLocationProblems {
        /// There are locations in the backing store that are not available in the database.
        pub missing_locations: Option<Vec<Location>>,
        /// There are locations in the database that are not available in the backing store.
        pub detached_locations: Option<Vec<Location>>,
    }

    /// Problems that were found with the database weather history data.
    ///
    #[derive(Debug, Default)]
    pub struct DbHistoryProblems {
        /// The database and filesystem have different history counts.
        pub history_problems: Option<Vec<DbHistoryProblemDetails>>,
        /// The database location does not have a corresponding filesystem location.
        pub detached_store: Option<Vec<HistorySummaries>>,
    }

    /// Detail information concerning problems with the data weather history data.
    ///
    #[derive(Debug)]
    pub struct DbHistoryProblemDetails {
        /// Identifies the location with problems.
        pub location: Location,
        /// The count of weather history data in the database.
        pub db_histories: usize,
        /// The count of weather history data in the backing store.
        pub fs_histories: usize,
    }

    /// Problems encountered when accessing the backing store locations document.
    ///
    #[derive(Debug, Default)]
    pub struct FilesysDocumentProblem {
        /// There was a problem trying to open the locations document.
        pub open_error: Option<crate::Error>,
        /// There was a problem trying to read the locations document.
        pub read_error: Option<crate::Error>,
    }
    impl FilesysDocumentProblem {
        /// Create a new instance when there is an open error.
        ///
        /// # Arguments
        ///
        /// * `error` has a description of the problem.
        ///
        pub fn open_error(error: crate::Error) -> Self {
            Self { open_error: Some(error), read_error: None }
        }

        /// Create a new instance when there is a read error.
        ///
        /// # Arguments
        ///
        /// * `error` has a description of the problem.
        ///
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

    /// The details about the Cities database.
    ///
    #[derive(Debug)]
    pub struct CitiesDetails {
        /// The database size in bytes.
        pub db_size: usize,
        /// The collection of details about countries.
        pub country_details: Vec<CountryDetails>,
    }

    /// The details about countries in the Cities database.
    ///
    #[derive(Debug)]
    pub struct CountryDetails {
        /// The name of the country.
        pub name: String,
        /// The country code name.
        pub code: String,
        /// The collection of details about regions in the country.
        pub region_details: Vec<RegionDetails>,
    }
    impl std::fmt::Display for CountryDetails {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} ({})", self.name, self.code)
        }
    }

    /// The details about a region in the Cities database.
    ///
    #[derive(Debug)]
    pub struct RegionDetails {
        /// The name of the region.
        pub name: String,
        /// The region code name.
        pub code: String,
        /// The count of cities in the region
        pub city_count: usize,
    }
}
