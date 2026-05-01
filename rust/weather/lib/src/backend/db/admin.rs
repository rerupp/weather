//! The weather data administration database API.

use super::sqlite;
use crate::{
    admin_prelude::{CitiesDetails, DbDetails, DbProblems},
    backend::filesys::WeatherDir,
    prelude::LocationFilter,
};
use std::rc::Rc;

/// Create the database administration API.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
///
pub(in crate::backend) fn create_db_admin(weather_dir: Rc<WeatherDir>) -> impl DbAdmin {
    sqlite::admin::SQLiteAdmin::new(weather_dir)
}

/// The database administration API.
///
pub(crate) trait DbAdmin {
    /// Initialize the weather history database schema.
    ///
    /// # Arguments
    ///
    /// * `update` when true will reapply the schema update regardless if it already appears to exist.
    ///
    fn history_init(&self, update: bool) -> crate::Result<bool>;

    /// Deletes the current database schema.
    ///
    /// # Arguments
    ///
    /// * `delete` when true will remove the database file.
    ///
    fn history_drop(&self, delete: bool) -> crate::Result<()>;

    /// Bulk load locations weather history into a pristine database.
    ///
    /// # Arguments
    ///
    /// * `threads` determines how many threads can be used by the loader.
    ///
    fn history_load(&self, threads: usize) -> crate::Result<()>;

    /// Return information about the weather history database.
    ///
    fn history_details(&self) -> crate::Result<Option<DbDetails>>;

    /// Reload metadata and history for locations.
    ///
    /// # Arguments
    ///
    /// * `repair` when true will try to fix problems that were found.
    ///
    fn history_check(&self, repair: bool) -> Option<DbProblems>;

    /// Reload metadata and history for locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations that will be reloaded.
    ///
    fn history_reload(&self, filters: Vec<LocationFilter>) -> crate::Result<usize>;

    /// Initialize the US cities database.
    ///
    fn cities_init(&self) -> crate::Result<()>;

    /// Delete the Cities database.
    ///
    fn cities_delete(&self) -> crate::Result<()>;

    /// Load the Cities database.
    ///
    /// # Arguments
    ///
    /// * `csv_database` is the filename with the country cities CSV database.
    /// * `reload` will remove the country cities if one has been previously loaded.
    ///
    fn cities_load(&self, uscities_path: &str, reload: bool) -> crate::Result<usize>;

    /// Retrieve details about the Cities database.
    fn cities_details(&self) -> crate::Result<Option<CitiesDetails>>;
}
