//! The weather data administration database API.

use super::sqlite;
use crate::{
    admin::{DbDetails, UsCityDetails},
    backend::filesys::WeatherDir,
    entities::LocationFilters,
};
use std::path::PathBuf;

/// Initialize the database schema.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
/// * `db_mode` is the database configuration to initialize.
/// * `drop` when true will delete the schema before initialization.
/// * `load` when true will load weather data into the database.
///
pub fn init_db(weather_dir: &WeatherDir, drop: bool, load: bool, threads: usize) -> crate::Result<()> {
    sqlite::admin::init_db(weather_dir, drop, load, threads)
}

/// Deletes the current database schema.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
/// * `delete` when true will remove the database file.
///
pub fn drop_db(weather_dir: &WeatherDir, delete: bool) -> crate::Result<()> {
    sqlite::admin::drop_db(weather_dir, delete)
}

/// Provide information about the database.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
///
pub fn db_details(weather_dir: &WeatherDir) -> crate::Result<Option<DbDetails>> {
    sqlite::admin::db_details(weather_dir)
}

/// Reload metadata and history for locations.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
/// * `filters` identifies the locations that will be reloaded.
///
pub fn reload(weather_dir: &WeatherDir, filters: LocationFilters) -> crate::Result<Vec<String>> {
    sqlite::admin::reload(weather_dir, filters)
}

/// Creates the database counting the US Cities `CSV` file.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
/// *`csv_file` is the US Cities `CSV` file to load.
///
// todo: change the signature to take a str
pub fn uscities_load(weather_dir: &WeatherDir, csv_file: &PathBuf) -> crate::Result<usize> {
    sqlite::admin::uscities_load(weather_dir, csv_file.display().to_string().as_str())
}

/// Delete the US Cities database.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
///
pub fn uscities_delete(weather_dir: &WeatherDir) -> crate::Result<()> {
    sqlite::admin::uscities_delete(weather_dir)
}

/// Show information about the US Cities database.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
///
pub fn uscities_info(weather_dir: &WeatherDir) -> crate::Result<UsCityDetails> {
    sqlite::admin::uscities_info(weather_dir)
}
