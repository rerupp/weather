mod history_loader;

use super::{history, locations, prepare_sql, query_rows, us_cities};
use crate::{
    admin::{DbDetails, LocationDetails, UsCityDetails},
    backend::filesys::WeatherDir,
    entities::LocationFilters,
};
use rusqlite::{Connection, Row};

/// Create a database history specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("SQLite admin {}", format!($($arg)*)))
    }
}

/// Create an error from history specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// Initialize the database.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
/// * `db_mode` is the database configuration to initialize.
/// * `drop` when true will delete the schema before initialization.
/// * `load` when true will load weather data into the database.
///
pub fn init_db(weather_dir: &WeatherDir, drop: bool, load: bool, threads: usize) -> crate::Result<()> {
    if drop {
        drop_db(weather_dir, false)?;
    }
    let mut conn = super::db_conn!(weather_dir)?;
    init_schema(&conn)?;
    if load {
        log::debug!("loading data");
        locations::load(&mut conn, weather_dir)?;
        history_loader::load(conn, weather_dir, threads)?;
    }
    Ok(())
}

/// Initialize the database schema.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `db_mode` is the database configuration.
fn init_schema(conn: &Connection) -> crate::Result<()> {
    log::debug!("init schema");
    let sql = include_str!("schema.sql");
    if let Err(error) = conn.execute_batch(sql) {
        err!("failed to initialize the schema: {:?}", error)?;
    }
    Ok(())
}

/// Provide information about the database.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
///
pub fn db_details(weather_dir: &WeatherDir) -> crate::Result<Option<DbDetails>> {
    let mut db_details = None;
    let file = weather_dir.file(super::DB_FILENAME);
    if file.exists() {
        // query the db details
        let conn = super::db_conn!(weather_dir)?;
        const SQL: &str = r#"
            SELECT l.alias as alias, SUM(m.size) AS size, COUNT(*) AS histories
            FROM metadata AS m
                INNER JOIN locations AS l ON m.lid = l.id
            GROUP BY alias
        "#;
        let mut stmt = prepare_sql!(conn, SQL, "failed to prepare db details query")?;
        let mut rows = query_rows!(stmt, [], "failed to get db details")?;

        // get the results
        let mut location_details = vec![];
        loop {
            match rows.next() {
                Err(error) => err!("failed to get next db details row: {:?}", error)?,
                Ok(None) => break,
                Ok(Some(row)) => {
                    // mine the row data
                    #[inline]
                    fn next_details(row_: &Row) -> super::SqlResult<(String, usize, usize)> {
                        Ok((row_.get(0)?, row_.get(1)?, row_.get(2)?))
                    }
                    match next_details(row) {
                        Err(error) => err!("failed to get db details from row: {:?}", error)?,
                        Ok((alias, size, histories)) => {
                            location_details.push(LocationDetails { alias, size, histories });
                        }
                    }
                }
            };
        }
        db_details.replace(DbDetails { size: file.size() as usize, location_details });
    }
    Ok(db_details)
}

/// Deletes the current database schema.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
/// * `delete` when true will remove the database file.
///
pub fn drop_db(weather_dir: &WeatherDir, delete: bool) -> crate::Result<()> {
    let file = weather_dir.file(super::DB_FILENAME);
    if file.exists() {
        match delete {
            true => file.remove()?,
            false => drop_schema(super::db_conn!(weather_dir)?)?,
        }
    }
    Ok(())
}

/// Delete the database schema.
///
/// Arguments
///
/// * `conn` is the database connection that will be used.
///
fn drop_schema(conn: Connection) -> crate::Result<()> {
    log::debug!("drop schema");
    let sql = include_str!("drop.sql");

    // delete the existing schema
    if let Err(error) = conn.execute_batch(sql) {
        err!("failed to drop the existing schema: {:?}", error)?;
    } else if let Err(error) = conn.execute("VACUUM", ()) {
        err!("failed to repack database: {:?}", error)?;
    }
    Ok(())
}

/// Reload metadata and history for locations.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
/// * `filters` identifies the locations that will be reloaded.
///
pub fn reload(weather_dir: &WeatherDir, filters: LocationFilters) -> crate::Result<Vec<String>> {
    let mut reloaded = vec![];
    let mut conn = super::db_conn!(weather_dir)?;
    for location in locations::get(&conn, filters)? {
        history::reload(&mut conn, weather_dir, &location.alias)?;
        reloaded.push(location.alias);
    }
    Ok(reloaded)
}

/// Creates the database counting the US Cities `CSV` file.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
/// * `csv_file` is the US Cities `CSV` file to load.
///
pub fn uscities_load(weather_dir: &WeatherDir, csv_file: &str) -> crate::Result<usize> {
    if us_cities::exists(weather_dir) {
        err!("The US Cities database already exists.")?;
    }
    us_cities::create(weather_dir, csv_file)?;
    let count = us_cities::db_metrics(weather_dir)?.state_info.into_iter().map(|(_, count)| count).sum();
    Ok(count)
}

/// Delete the US Cities database.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
pub fn uscities_delete(weather_dir: &WeatherDir) -> crate::Result<()> {
    us_cities::delete(weather_dir)
}

/// Retrieve information about the US Cities database.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
///
pub fn uscities_info(weather_dir: &WeatherDir) -> crate::Result<UsCityDetails> {
    us_cities::db_metrics(weather_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{db::sqlite::DB_FILENAME, testlib};
    use std::path::PathBuf;

    #[test]
    fn admin() {
        let fixture = testlib::TestFixture::create();
        let test_files = testlib::test_resources().join("db");
        fixture.copy_resources(&test_files);
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let db_file = PathBuf::from(&weather_dir.to_string()).join(DB_FILENAME);
        assert!(!db_file.exists());
        init_db(&weather_dir, false, false, 1).unwrap();
        assert!(db_file.exists());
        db_details(&weather_dir).unwrap().expect("Did not get DbDetails");
        drop_db(&weather_dir, false).unwrap();
    }
}
