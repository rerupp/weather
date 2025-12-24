//! Encapsulates reading [simple maps](https://simplemaps.com/data/us-cities) US cities
//! CSV database.

pub(super) mod admin;
mod query;
pub use query::{cities as get_cities, states as get_states};

use super::db_connection;
use crate::backend::filesys::WeatherDir;
use rusqlite::Connection;

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
    let db_file = weather_dir.file(DB_FILENAME);
    db_file.exists() && db_file.size() > 0
}

/// Open the US Cities database.
///
pub fn open(weather_dir: &WeatherDir) -> crate::Result<Connection> {
    if !exists(weather_dir) {
        err!("{} has not been created.", DB_FILENAME)
    } else {
        match db_connection(Some(&weather_dir.file(DB_FILENAME))) {
            Ok(conn) => Ok(conn),
            Err(error) => err!(" could not open {}: {:?}", DB_FILENAME, error),
        }
    }
}

