//! The Sqlite database implementation for weather data.

pub mod admin;
mod cities;
mod history;
mod locations;
mod metadata;

use crate::{
    backend::{
        filesys::{WeatherDir, WeatherFile},
        Backend,
    },
    configuration::Configuration,
    entities::{City, DailyHistories, DateRange, HistoryDates, HistorySummaries, Location, LocationFilter},
};
use std::sync::Arc;

/// The result of a rusqlite function.
type SqlResult<T> = Result<T, rusqlite::Error>;

/// The name of the database
const DB_FILENAME: &str = "weather_data.db";

/// Create an error from the locations specific error message.
///
/// # Params
///
/// * `args` are passed to `format!` to create the error message.
///
macro_rules! err {
    ($($args:tt)*) => {
        Err(crate::Error(format!("SQLite {}", format!($($args)*))))
    };
}
use err;

/// Create a database connection.
///
/// # Arguments
///
/// * `optional_file` is the database file, if `None` an in-memory database will be used.
///
fn db_connection(optional_file: Option<&WeatherFile>) -> crate::Result<rusqlite::Connection> {
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
///
/// # Params
///
/// * `weather_dir` is the weather data directory.
///
macro_rules! db_conn {
    ($weather_dir:expr) => {
        $crate::backend::db::sqlite::db_connection(Some(&$weather_dir.file(crate::backend::db::sqlite::DB_FILENAME)))
    };
}
use db_conn;

/// A helper to execute SQL.
///
/// # Params
///
/// * `stmt` is the SQL statement that will be run.
/// * `params` holds the SQL statement parameters.
/// * `args` are passed to `format!` if there is an error.
///
macro_rules! execute_sql {
    ($stmt:expr, $params:expr, $($arg:tt)*) => {
        match $stmt.execute($params) {
            Ok(updates) => Ok(updates),
            Err(error) => err!("{}: {:?}", format!($($arg)*), error)
        }
    };
}
use execute_sql;

/// A helper to prepare an SQL statement.
///
/// # Params
///
/// * `conn` is the database connection
/// * `sql` is the sql
/// * `args` are passed to `format!` if there is an error.
///
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
///
/// # Params
///
/// * `conn` is the database connection
/// * `sql` is the sql
/// * `args` are passed to `format!` if there is an error.
///
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
///
/// #Params
///
/// * `stmt` is the statement.
/// * `params` are the SQL parameters.
/// * `args` are used the `format!` to create an error description.
///
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
///
/// # Params
///
/// * `conn` is the database connection
/// * `args` are passed to `format!` if there is an error.
///
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
///
/// # Params
///
/// * `tx` is the database transaction.
/// * `args` are passed to `format!` if there is an error.
///
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
    /// The weather data directory.
    weather_dir: WeatherDir,
}
impl SqliteBackend {
    /// Create a new instance of the sqlite backend.
    ///
    /// # Arguments
    ///
    /// * `configuration` is the current weather data configuration.
    ///
    pub fn new(configuration: Arc<Configuration>) -> crate::Result<Self> {
        log::debug!("SqliteBackend");
        let weather_dir = WeatherDir::try_from(&configuration)?;
        Ok(Self { weather_dir })
    }

    /// Get a location.
    ///
    /// # Arguments
    ///
    /// * `conn` is the database connection that will be used.
    /// * `filter` is used to get the location.
    ///
    fn get_location(&self, conn: &rusqlite::Connection, filter: LocationFilter) -> crate::Result<Option<Location>> {
        let mut locations = locations::get(conn, Some(vec![filter]))?;
        match locations.len() {
            0 => Ok(None),
            1 => Ok(locations.pop()),
            _ => err!("Multiple locations were found."),
        }
    }
}
impl Backend for SqliteBackend {
    /// Add weather data history to a location.
    ///
    /// # Arguments
    ///
    /// - `daily_histories` contains the historical weather data that will be added.
    ///
    fn add_daily_histories(&self, daily_histories: DailyHistories) -> crate::Result<usize> {
        let mut conn = db_conn!(&self.weather_dir)?;
        history::add(&mut conn, &self.weather_dir, daily_histories)
    }

    /// Get daily weather history for a location.
    ///
    /// It is an error if more than 1 location is found.
    ///
    /// # Arguments
    ///
    /// - `filter` identifies the location.
    /// - `history_range` covers the history dates returned.
    ///
    fn get_daily_histories(&self, filter: LocationFilter, history_range: DateRange) -> crate::Result<DailyHistories> {
        let mut conn = db_conn!(&self.weather_dir)?;
        match self.get_location(&conn, filter)? {
            None => err!("The location was not found."),
            Some(location) => history::get(&mut conn, location, history_range),
        }
    }

    /// Get the history dates for locations.
    ///
    /// # Arguments
    ///
    /// - `filters` identifies the locations.
    ///
    fn get_history_dates(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistoryDates>> {
        let conn = db_conn!(&self.weather_dir)?;
        history::history_dates(&conn, filters)
    }

    /// Get a summary of location weather data.
    ///
    /// # Arguments
    ///
    /// - `filters` identifies the optional locations of interest.
    ///
    fn get_history_summaries(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistorySummaries>> {
        let mut conn = db_conn!(&self.weather_dir)?;
        history::summary(&mut conn, &self.weather_dir, filters)
    }

    /// Get the weather location metadata.
    ///
    /// # Arguments
    ///
    /// - `filters` identifies the optional locations of interest.
    ///
    fn get_locations(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<Location>> {
        let conn = db_conn!(&self.weather_dir)?;
        locations::get(&conn, filters)
    }

    /// Add a location.
    ///
    /// #Arguments
    ///
    /// * `location` is the location data.
    ///
    fn add_location(&self, location: Location) -> crate::Result<()> {
        let mut conn = db_conn!(&self.weather_dir)?;
        locations::add(&mut conn, location, &self.weather_dir)
    }

    /// Delete a location.
    ///
    /// # Arguments
    ///
    /// * `filter` is used to identify a location.
    ///
    fn delete_location(&self, filter: LocationFilter) -> crate::Result<()> {
        let mut conn = db_conn!(&self.weather_dir)?;
        match self.get_location(&conn, filter)? {
            None => err!("The location was not found."),
            Some(location) => {
                let tx = create_tx!(conn, "Failed to create delete tx for {location}.")?;
                let lid = locations::location_id(&tx, &location.alias)?;
                history::delete(&tx, lid)?;
                metadata::delete(&tx, lid)?;
                locations::delete(&tx, &location.alias, &self.weather_dir)?;
                commit_tx!(tx, "Error deleting {location}")
            }
        }
    }

    /// Update a location properties.
    ///
    /// # Arguments
    ///
    /// * `location` identifies the location and contains the new property values.
    ///
    fn update_location(&self, location: Location) -> crate::Result<bool> {
        let mut conn = db_conn!(&self.weather_dir)?;
        locations::update(&mut conn, location, &self.weather_dir)
    }

    /// Search for cities.
    ///
    /// # Arguments
    ///
    /// * `filters` is used to identify the cities.
    /// * `limit` restricts the number of cities returned.
    ///
    fn get_cities(&self, filters: Option<Vec<LocationFilter>>, limit: usize) -> crate::Result<Vec<City>> {
        if !cities::is_initialized(&self.weather_dir) {
            err!("The Cities database has not been initialized.")?;
        }
        cities::get(&self.weather_dir, filters, limit)
    }
}

/// Tests if the database file exists or not.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
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
///
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

pub fn sql_match_condition(column: impl ToString, filter: impl ToString) -> crate::Result<String> {
    let mut filter = filter.to_string();
    let column = column.to_string();

    // if there are illegal characters discard the filter
    if filter.contains(|c| r"[]^!\".contains(c)) {
        err!("The column '{column}' match value '{filter}' contains illegal characters.")?;
    }

    // check if there are sql wildcards
    let is_sql_glob_wildcard = filter.contains(|c| c == '%');
    if is_sql_glob_wildcard {
        filter = filter.replace("%", r"\%");
    }
    let is_sql_char_wildcard = filter.contains(|c| c == '_');
    if is_sql_char_wildcard {
        filter = filter.replace("_", r"\_");
    }

    // fix up any of the filter wildcards
    let is_glob_wildcard = filter.contains(|c| c == '*');
    if is_glob_wildcard {
        filter = filter.replace("*", "%");
    }
    let is_char_wildcard = filter.contains(|c| c == '.');
    if is_char_wildcard {
        filter = filter.replace(".", "_");
    }

    let sql = match is_glob_wildcard || is_char_wildcard {
        false => match is_sql_glob_wildcard || is_sql_char_wildcard {
            true => format!("{column} = '{filter}' ESCAPE '\\'"),
            false => format!("{column} = '{filter}'"),
        },
        true => match is_sql_glob_wildcard || is_sql_char_wildcard {
            true => format!("{column} LIKE '{filter}' ESCAPE '\\'"),
            false => format!("{column} LIKE '{filter}'"),
        },
    };
    Ok(sql)
}

#[cfg(test)]
mod tests {
    #[test]
    fn sql_match_condition() {
        // the illegal characters
        assert!(super::sql_match_condition("column", r"[").is_err());
        assert!(super::sql_match_condition("column", r"]").is_err());
        assert!(super::sql_match_condition("column", r"^").is_err());
        assert!(super::sql_match_condition("column", r"!").is_err());
        assert!(super::sql_match_condition("column", r"\").is_err());

        let testcase = |column: &str, value: &str| super::sql_match_condition(column, value).unwrap();

        assert_eq!(testcase("column", ""), "column = ''");

        assert_eq!(testcase("column", "foo"), "column = 'foo'");
        assert_eq!(testcase("column", "*foo"), "column LIKE '%foo'");
        assert_eq!(testcase("column", "foo*"), "column LIKE 'foo%'");
        assert_eq!(testcase("column", "*foo*"), "column LIKE '%foo%'");
        assert_eq!(testcase("column", ".foo"), "column LIKE '_foo'");
        assert_eq!(testcase("column", "foo."), "column LIKE 'foo_'");
        assert_eq!(testcase("column", ".foo."), "column LIKE '_foo_'");

        assert_eq!(testcase("column", "foo_bar"), r"column = 'foo\_bar' ESCAPE '\'");
        assert_eq!(testcase("column", "*foo_bar."), r"column LIKE '%foo\_bar_' ESCAPE '\'");
    }
}
