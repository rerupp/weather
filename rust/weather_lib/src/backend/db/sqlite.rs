//! The Sqlite database implementation of weather data.
//!
//! The implementation uses the [filesystem](crate::backend::filesys) as a backing store.
//! Data is added to the filesystem prior to being added to the database. This provides
//! a simple way to create a backup of existing weather history. It also facilitates an
//! easy way to reload history data after changes are made to the database schema.
//!

pub mod admin;
mod cities;
pub mod tables;
pub mod weather;

use crate::{
    backend::{
        filesys::{WeatherDir, WeatherFile},
        Backend,
    },
    configuration::Configuration,
    entities::{City, DailyHistories, DateRange, HistoryDates, HistorySummary, Location, LocationFilter},
};
use std::sync::Arc;

#[doc(hidden)]
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
/// * `file_opt` is the database file, if `None` an in-memory database will be used.
///
fn db_connection(file_opt: Option<&WeatherFile>) -> crate::Result<rusqlite::Connection> {
    match file_opt {
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

/// Query the database and call the `read` function for each row in the result set.
///
/// # Arguments
///
/// * `stmt` is the  query that will be run.
/// * `params` holds the query parameters.
/// * `read` is the function called for each row in the result set.
///
fn query_rows<P, F>(mut stmt: rusqlite::Statement, params: P, mut read: F) -> crate::Result<()>
where
    P: rusqlite::Params,
    F: FnMut(&rusqlite::Row) -> crate::Result<()>,
{
    match stmt.query(params) {
        Err(error) => err!("The reader query resulted in an error: {error:?}"),
        Ok(mut rows) => {
            loop {
                match rows.next() {
                    Err(error) => err!("There was a reader error getting the next row: {error:?}")?,
                    Ok(None) => break,
                    Ok(Some(row)) => read(row)?,
                }
            }
            Ok(())
        }
    }
}

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
            Err(error) => err!("{}.\n{:?}", format!($($arg)*), error)
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
}
impl Backend for SqliteBackend {
    /// Add weather data history to a location.
    ///
    /// # Arguments
    ///
    /// - `daily_histories` contains the historical weather data that will be added.
    ///
    fn add_daily_histories(&self, daily_histories: DailyHistories) -> crate::Result<usize> {
        let mut conn = weather::db_conn!(&self.weather_dir)?;
        weather::history::add(&mut conn, &self.weather_dir, daily_histories)
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
        let mut conn = weather::db_conn!(&self.weather_dir)?;
        match weather::locations::get_one(&conn, filter)? {
            None => err!("The location was not found."),
            Some(location) => weather::history::get(&mut conn, location, history_range),
        }
    }

    /// Get the history dates for locations.
    ///
    /// # Arguments
    ///
    /// - `filters` identifies the locations.
    ///
    fn get_history_dates(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistoryDates>> {
        let conn = weather::db_conn!(&self.weather_dir)?;
        weather::history::history_dates(&conn, filters)
    }

    /// Get a summary of location weather data.
    ///
    /// # Arguments
    ///
    /// - `filters_opt` identifies the optional locations of interest.
    ///
    fn get_history_summaries(&self, filters_opt: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistorySummary>> {
        let mut conn = weather::db_conn!(&self.weather_dir)?;
        weather::history::summary(&mut conn, &self.weather_dir, filters_opt)
    }

    /// Get the weather location metadata.
    ///
    /// # Arguments
    ///
    /// - `filters` identifies the optional locations of interest.
    ///
    fn get_locations(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<Location>> {
        let conn = weather::db_conn!(&self.weather_dir)?;
        weather::locations::get(&conn, filters)
    }

    /// Add a location.
    ///
    /// #Arguments
    ///
    /// * `location` is the location data.
    ///
    fn add_location(&self, location: Location) -> crate::Result<()> {
        let mut conn = weather::db_conn!(&self.weather_dir)?;
        weather::locations::add(&mut conn, location, &self.weather_dir)
    }

    /// Delete a location.
    ///
    /// # Arguments
    ///
    /// * `filter` is used to identify a location.
    ///
    fn delete_location(&self, filter: LocationFilter) -> crate::Result<()> {
        let mut conn = weather::db_conn!(&self.weather_dir)?;
        weather::delete_location(&mut conn, &self.weather_dir, filter)
    }

    /// Update a location properties.
    ///
    /// # Arguments
    ///
    /// * `location` identifies the location and contains the new property values.
    ///
    fn update_location(&self, location: Location) -> crate::Result<bool> {
        let mut conn = weather::db_conn!(&self.weather_dir)?;
        weather::locations::update(&mut conn, location, &self.weather_dir)
    }

    /// Search for cities.
    ///
    /// # Arguments
    ///
    /// * `filters` is used to identify the cities.
    /// * `limit` restricts the number of cities returned.
    ///
    fn get_cities(&self, filters: Option<Vec<LocationFilter>>, limit: usize) -> crate::Result<Vec<City>> {
        cities::get(&self.weather_dir, filters, limit)
    }
}

/// Generate the SQL fragment `column='filter'`, `column LIKE 'filter'`, or `column LIKE 'filter' ESCAPE '\'`.
///
/// # Arguments
///
/// * `column` is the table column name.
/// * `filter` will be converted to SQL matching syntax instead of `grep` like syntax.
///
pub fn generate_sql_match_condition(column: impl ToString, filter: impl ToString) -> crate::Result<String> {
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
            true => format!("{column} LIKE '{filter}' ESCAPE '\\'"),
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
        assert!(super::generate_sql_match_condition("column", r"[").is_err());
        assert!(super::generate_sql_match_condition("column", r"]").is_err());
        assert!(super::generate_sql_match_condition("column", r"^").is_err());
        assert!(super::generate_sql_match_condition("column", r"!").is_err());
        assert!(super::generate_sql_match_condition("column", r"\").is_err());

        let testcase = |column: &str, value: &str| super::generate_sql_match_condition(column, value).unwrap();

        assert_eq!(testcase("column", ""), "column = ''");

        assert_eq!(testcase("column", "foo"), "column = 'foo'");
        assert_eq!(testcase("column", "*foo"), "column LIKE '%foo'");
        assert_eq!(testcase("column", "foo*"), "column LIKE 'foo%'");
        assert_eq!(testcase("column", "*foo*"), "column LIKE '%foo%'");
        assert_eq!(testcase("column", ".foo"), "column LIKE '_foo'");
        assert_eq!(testcase("column", "foo."), "column LIKE 'foo_'");
        assert_eq!(testcase("column", ".foo."), "column LIKE '_foo_'");

        assert_eq!(testcase("column", "foo_bar"), r"column LIKE 'foo\_bar' ESCAPE '\'");
        assert_eq!(testcase("column", "*foo_bar."), r"column LIKE '%foo\_bar_' ESCAPE '\'");
    }
}
