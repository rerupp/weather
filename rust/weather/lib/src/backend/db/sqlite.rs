//! The Sqlite database implementation for weather data.

pub mod admin;
mod history;
mod locations;
mod metadata;
// you need to expose this for filesys right now.
pub mod us_cities;

use super::LocationFilters;
use crate::{
    backend::{
        filesys::{WeatherDir, WeatherFile},
        Backend, Config,
    },
    entities::{DailyHistories, DateRange, HistoryDates, HistorySummaries, Location, State, CityFilter},
};

/// The result of some rusqlite function.
type SqlResult<T> = Result<T, rusqlite::Error>;

/// The name of the database
const DB_FILENAME: &str = "weather_data.db";

/// Create a database locations specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("SQLite {}", format!($($arg)*)))
    }
}
use error;

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err($crate::backend::db::sqlite::error!($($arg)*))
    };
}
use err;

/// Create a database connection.
///
/// # Arguments
///
/// * `optional_file` is the database file, if `None` an in-memory database will be used.
///
pub(in crate::backend::db) fn db_connection(optional_file: Option<WeatherFile>) -> crate::Result<rusqlite::Connection> {
    match optional_file {
        Some(file) => match rusqlite::Connection::open(file.to_string()) {
            Ok(conn) => Ok(conn),
            Err(error) => err!("failed to get a database connection to {}: {:?}", file, error),
        },
        None => match rusqlite::Connection::open_in_memory() {
            Ok(conn) => Ok(conn),
            Err(error) => err!("failed to create in-memory database connection: {:?}", error),
        },
    }
}

/// A helper to create a database connection.
macro_rules! db_conn {
    ($weather_dir:expr) => {
        $crate::backend::db::sqlite::db_connection(Some($weather_dir.file(crate::backend::db::sqlite::DB_FILENAME)))
    };
}
use db_conn;

/// A helper to execute SQL.
macro_rules! execute_sql {
    ($stmt:expr, $params:expr, $($arg:tt)*) => {
        match $stmt.execute($params) {
            Ok(_) => Ok(()),
            Err(error) => err!("{}: {:?}", format!($($arg)*), error)
        }
    };
}
use execute_sql;

/// A helper to prepare an SQL statement.
macro_rules! prepare_sql {
    ($conn:expr, $sql:expr, $($args:tt)*) => {
        match $conn.prepare($sql) {
            Ok(stmt) => Ok(stmt),
            Err(error) =>err!("{}: {:?}", format!($($args)*), error)
        }
    };
}
use prepare_sql;

/// A helper to prepare a cached SQL statement.
macro_rules! prepare_cached_sql {
    ($conn:expr, $sql:expr, $($args:tt)*) => {
        match $conn.prepare_cached($sql) {
            Ok(stmt) => Ok(stmt),
            Err(error) => err!("{}: {:?}", format!($($args)*), error)
        }
    };
}
use prepare_cached_sql;

/// A helper to query rows from the database.
macro_rules! query_rows {
    ($stmt:expr, $params:expr, $($args:tt)*) => {
        match $stmt.query($params) {
            Ok(rows) => Ok(rows),
            Err(error) => err!("{}: {:?}", format!($($args)*), error)
        }
    };
}
use query_rows;

/// A helper that creates a transaction.
macro_rules! create_tx {
    ($conn:expr, $($args:tt)*) => {
        match $conn.transaction() {
            Ok(tx) => Ok(tx),
            Err(error) => err!("{}: {:?}", format!($($args)*), error)
        }
    };
}
use create_tx;

/// A helper that commits a transaction.
macro_rules! commit_tx {
    ($tx:expr, $($arg:tt)*) => {
        match $tx.commit() {
            Ok(_) => Ok(()),
            Err(error) => err!("{}: {:?}", format!($($arg)*), error)
        }
    };
}
use commit_tx;

/// The Sqlite3 database data adapter implementation.
pub struct SqliteBackend {
    /// The weather data configuration being used.
    config: Config,
    /// The weather data directory.
    weather_dir: WeatherDir,
}
impl SqliteBackend {
    pub fn new(config: Config, weather_dir: WeatherDir) -> Self {
        Self { config, weather_dir }
    }
}
impl Backend for SqliteBackend {
    fn get_config(&self) -> &Config {
        &self.config
    }

    fn add_daily_histories(&self, daily_histories: DailyHistories) -> crate::Result<usize> {
        let mut conn = db_conn!(&self.weather_dir)?;
        history::add(&mut conn, &self.weather_dir, daily_histories)
    }

    fn get_daily_histories(&self, filters: LocationFilters, history_range: DateRange) -> crate::Result<DailyHistories> {
        let mut conn = db_conn!(&self.weather_dir)?;
        let mut locations = locations::get(&conn, filters)?;
        let location = match locations.len() {
            1 => locations.pop().unwrap(),
            0 => err!("a location was not found.")?,
            _ => err!("Multiple locations were found.")?,
        };
        history::get(&mut conn, location, history_range)
    }

    fn get_history_dates(&self, filters: LocationFilters) -> crate::Result<Vec<HistoryDates>> {
        let conn = db_conn!(&self.weather_dir)?;
        history::history_dates(&conn, filters)
    }

    fn get_history_summaries(&self, filters: LocationFilters) -> crate::Result<Vec<HistorySummaries>> {
        let mut conn = db_conn!(&self.weather_dir)?;
        history::summary(&mut conn, &self.weather_dir, filters)
    }

    fn get_locations(&self, filters: LocationFilters) -> crate::Result<Vec<Location>> {
        let conn = db_conn!(&self.weather_dir)?;
        locations::get(&conn, filters)
    }

    fn add_location(&self, location: Location) -> crate::Result<()> {
        let mut conn = db_conn!(&self.weather_dir)?;
        locations::add(&mut conn, location, &self.weather_dir)
    }

    fn search_locations(&self, filter: CityFilter) -> crate::Result<Vec<Location>> {
        if !us_cities::exists(&self.weather_dir) {
            us_cities::create(&self.weather_dir, &self.config.us_cities.filename)?;
        }
        us_cities::get_cities(&us_cities::open(&self.weather_dir)?, filter)
    }

    fn get_states(&self) -> crate::Result<Vec<State>> {
        us_cities::get_states(&us_cities::open(&self.weather_dir)?)
    }
}

/// Tests if the database file exists or not.
///
pub fn db_exists(weather_dir: &WeatherDir) -> bool {
    weather_dir.file(DB_FILENAME).exists()
}

/// Get the size estimate of a table in the database. This is specific to `sqlite`.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `table` is the database table name.
// todo: should this be somewhere else?
pub fn estimate_size(conn: &rusqlite::Connection, table: &str) -> crate::Result<usize> {
    let mut size_estimate = 0;
    let pragma_result: SqlResult<()> = conn.pragma(None, "table_info", table, |row| {
        let name: String = row.get("name")?;
        let column_type: String = row.get("type")?;
        match column_type.as_str() {
            "REAL" => size_estimate += 8,
            "INTEGER" => {
                if name.ends_with("_t") {
                    size_estimate += 8;
                } else if name == "id" || name == "mid" {
                    // primary ids are always 8 bytes
                    size_estimate += 8;
                } else {
                    size_estimate += 4;
                }
            }
            "TEXT" => (),
            _ => {
                eprintln!("Yikes!!!! Did not recognize column {} type '{}'...", name, column_type);
            }
        }
        Ok(())
    });
    if let Err(error) = pragma_result {
        err!("failed to estimate the size of {table}: {:?}", error)?;
    }
    Ok(size_estimate)
}
