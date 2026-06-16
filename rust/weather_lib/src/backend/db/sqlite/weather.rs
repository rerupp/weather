//! The SQLite weather history database implementation.
//!
pub mod history;
pub mod locations;

use rusqlite::Connection;

use crate::{
    backend::db::sqlite::{commit_tx, create_tx, err, weather, WeatherDir},
    entities::LocationFilter,
};

/// The name of the weather history database
pub const WEATHER_DB_FILENAME: &str = "weather.db";

/// A helper to create a database connection.
///
/// # Params
///
/// * `weather_dir` is the weather data directory.
///
macro_rules! db_conn {
    ($weather_dir:expr) => {
        $crate::backend::db::sqlite::db_connection(Some(
            &$weather_dir.file(crate::backend::db::sqlite::weather::WEATHER_DB_FILENAME),
        ))
    };
}
pub(super) use db_conn;

/// Tests if the database file exists or not.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
///
pub fn db_exists(weather_dir: &WeatherDir) -> bool {
    weather_dir.file(WEATHER_DB_FILENAME).exists()
}

/// Delete the database file.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
///
pub fn db_delete(weather_dir: &WeatherDir) -> bool {
    let file = weather_dir.file(WEATHER_DB_FILENAME);
    match file.exists() {
        false => true,
        true => match file.remove() {
            Ok(_) => true,
            Err(error) => {
                log::error!("There was an error deleting the database.\n{}", error);
                false
            }
        },
    }
}

/// Get the database file size.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
///
pub fn db_size(weather_dir: &WeatherDir) -> u64 {
    let file = weather_dir.file(WEATHER_DB_FILENAME);
    match file.exists() {
        false => 0,
        true => file.size(),
    }
}

/// Delete a location.
///
/// # Arguments
///
/// * `filter` is used to identify a location.
///
pub fn delete_location(conn: &mut Connection, weather_dir: &WeatherDir, filter: LocationFilter) -> crate::Result<()> {
    match locations::get_one(&conn, filter)? {
        None => err!("The location was not found."),
        Some(location) => {
            let mut tx = create_tx!(conn, "Failed to create delete tx for {location}.")?;
            let lid = weather::locations::location_id(&tx, &location.alias)?;
            history::delete(&mut tx, lid)?;
            locations::delete(&tx, &location.alias, weather_dir)?;
            commit_tx!(tx, "Error deleting {location}")
        }
    }
}
