//! Encapsulates reading [simple maps](https://simplemaps.com/data/us-cities) US cities
//! CSV database.

mod admin;
mod query;
pub use query::{cities as get_cities, states as get_states};

use super::db_connection;
use crate::{
    admin::UsCityDetails,
    backend::filesys::WeatherDir,
    // entities::{Location, State, CityFilter},
};
use rusqlite::Connection;
use std::path::PathBuf;

/// The default name of the US Cities database;
const DB_FILENAME: &'static str = "uscities.db";

/// Create a database locations specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("US Cities {}", format!($($arg)*)))
    }
}

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// Check if the US Cities database exists.
///
pub fn exists(weather_dir: &WeatherDir) -> bool {
    weather_dir.file(DB_FILENAME).exists()
}

/// Open the US Cities database.
///
pub fn open(weather_dir: &WeatherDir) -> crate::Result<Connection> {
    if !exists(weather_dir) {
        err!("{} has not been created.", DB_FILENAME)
    } else {
        match db_connection(Some(weather_dir.file(DB_FILENAME))) {
            Ok(conn) => Ok(conn),
            Err(error) => err!(" could not open {}: {:?}", DB_FILENAME, error),
        }
    }
}

/// Create the US Cities database.
///
pub fn create(weather_dir: &WeatherDir, csv_file: &str) -> crate::Result<()> {
    if exists(weather_dir) {
        err!("{} has already been created.", DB_FILENAME)?;
    }
    crate::log_elapsed_time!(info, "US Cities create");
    let mut conn = match db_connection(Some(weather_dir.file(DB_FILENAME))) {
        Ok(conn) => conn,
        Err(error) => err!(" could not create {}: {:?}", DB_FILENAME, error)?,
    };
    admin::init_schema(&conn)?;
    admin::load_db(&mut conn, PathBuf::from(csv_file))?;
    Ok(())
}

/// Delete the US Cities database
///
pub fn delete(weather_dir: &WeatherDir) -> crate::Result<()> {
    let file = weather_dir.file(DB_FILENAME);
    if file.exists() {
        if let Err(error) = file.remove() {
            err!("failed to delete database file: {:?}", error)?;
        }
    }
    Ok(())
}

pub fn db_metrics(weather_dir: &WeatherDir) -> crate::Result<UsCityDetails> {
    let file = weather_dir.file(DB_FILENAME);
    if file.exists() {
        let db_size = file.size() as usize;
        let conn = match db_connection(Some(file)) {
            Ok(conn) => conn,
            Err(error) => err!("did not get a db connection: {:?}", error)?,
        };
        let state_info = admin::state_metrics(&conn)?;
        Ok(UsCityDetails { db_size, state_info })
    } else {
        Ok(UsCityDetails { db_size: 0, state_info: Vec::with_capacity(0) })
    }
}
