mod history_loader;

use super::{commit_tx, create_tx, history, locations, prepare_sql, query_rows, us_cities::admin::UsCitiesAdmin};
use crate::{
    admin_prelude::{DbDetails, DbProblems, LocationDetails, UsCityDetails},
    backend::{db::admin::DbAdmin, filesys::{fs_lib, WeatherDir}},
    entities::LocationFilter,
};
use rusqlite::{params, Connection, Row};
use std::{fmt::Formatter, rc::Rc};

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

pub(in crate::backend::db) struct SQLiteAdmin {
    weather_dir: Rc<WeatherDir>,
}
impl std::fmt::Debug for SQLiteAdmin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SQLiteAdmin({})", self.weather_dir)
    }
}
impl SQLiteAdmin {
    pub fn new(weather_dir: Rc<WeatherDir>) -> Self {
        Self { weather_dir }
    }
    /// Check if the database has already been initialized.
    ///
    /// # Arguments
    ///
    /// * `conn` is the database connection that will be used by the query.
    ///
    fn is_initialized(&self, conn: &Connection) -> crate::Result<bool> {
        const SCHEMA_SQL: &str =
            "SELECT COUNT(*) FROM sqlite_schema WHERE tbl_name IN ('locations', 'metadata', 'history')";
        let mut stmt = conn.prepare(SCHEMA_SQL).unwrap();
        match stmt.query_row(params![], |row| row.get::<usize, usize>(0)) {
            Err(error) => err!("sqlite_schema query failed: {error}")?,
            Ok(count) => Ok(count > 0),
        }
    }
}
impl DbAdmin for SQLiteAdmin {
    /// Initialize the weather history database schema.
    ///
    /// # Arguments
    ///
    /// * `update` when true will initialize the history schema regardless if it exists.
    ///
    fn history_init(&self, update: bool) -> crate::Result<bool> {
        log::debug!("initializing history schema");
        let conn = super::db_conn!(&self.weather_dir)?;

        let initialize = update || !self.is_initialized(&conn)?;
        if initialize {
            let sql = include_str!("schema.sql");
            if let Err(error) = conn.execute_batch(sql) {
                err!("failed to initialize the schema: {:?}", error)?;
            }
        }
        Ok(initialize)
    }

    /// Deletes the current database schema.
    ///
    /// # Arguments
    ///
    /// * `delete` when true will remove the database file.
    ///
    fn history_drop(&self, delete: bool) -> crate::Result<()> {
        let file = self.weather_dir.file(super::DB_FILENAME);
        if file.exists() {
            match delete {
                true => file.remove()?,
                false => {
                    log::debug!("dropping history schema");
                    let conn = super::db_conn!(&self.weather_dir)?;
                    let sql = include_str!("drop.sql");
                    if let Err(error) = conn.execute_batch(sql) {
                        err!("failed to drop the existing schema: {:?}", error)?;
                    }
                    // you can't use VACUUM in a transaction
                    if let Err(error) = conn.execute("VACUUM", ()) {
                        err!("failed to repack database: {:?}", error)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Bulk load locations weather history into a pristine database.
    ///
    /// # Arguments
    ///
    /// * `threads` determines how many threads can be used by the loader.
    ///
    fn history_load(&self, threads: usize) -> crate::Result<()> {
        log::debug!("loading history data");
        let mut conn = super::db_conn!(&self.weather_dir)?;
        // todo: these should take a tx
        locations::load(&mut conn, &self.weather_dir)?;
        crate::log_elapsed_time!("history_loader");
        history_loader::load(conn, &self.weather_dir, threads)
    }

    /// Mine information about the weather history database.
    ///
    fn history_details(&self) -> crate::Result<Option<DbDetails>> {
        let mut db_details = None;
        let file = self.weather_dir.file(super::DB_FILENAME);
        if file.exists() {
            // query the db details
            let conn = super::db_conn!(&self.weather_dir)?;
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

    fn history_check(&self, _repair: bool) -> Option<DbProblems> {
        let mut conn = match super::db_conn!(&self.weather_dir) {
            Ok(conn) => conn,
            Err(error) => {
                return Some(DbProblems::from(error));
            }
        };
        let location_problems = match locations::check(&mut conn, &self.weather_dir) {
            Ok(location_problems) => location_problems,
            Err(error) => {
                return Some(DbProblems::from(error));
            }
        };
        let history_problems = match history::check(&mut conn, &self.weather_dir) {
            Ok(history_problems) => history_problems,
            Err(error) => {
                return Some(DbProblems::from(error));
            }
        };
        match location_problems.is_none() && history_problems.is_none() {
            true => None,
            false => {
                let mut db_problems = DbProblems::default();
                if let Some(location_problems) = location_problems {
                    db_problems.location_problems.replace(location_problems);
                }
                if let Some(history_problems) = history_problems {
                    db_problems.history_problems.replace(history_problems);
                }
                Some(db_problems)
            }
        }
    }

    /// Reload metadata and history for locations.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `filters` identifies the locations that will be reloaded.
    ///
    fn history_reload(&self, filters: Vec<LocationFilter>) -> crate::Result<usize> {
        let fs_locations = fs_lib::get_locations(&self.weather_dir, Some(filters.clone()))?;
        if fs_locations.is_empty() {
            log::warn!("There are no locations in the filesystem to reload.");
            return Ok(0);
        }

        // get locations from the database
        let mut conn = super::db_conn!(&self.weather_dir)?;
        let db_locations = locations::get(&conn, Some(filters.clone()))?;

        // refresh the db locations
        let tx = create_tx!(conn, "failed to create tx to add/update locations")?;
        for fs_location in &fs_locations {
            if !db_locations.iter().any(|db_location| fs_location.alias == db_location.alias) {
                locations::add_db(&tx, fs_location.clone())?;
                log::debug!("{} ({}) added.", fs_location.name, fs_location.alias);
            } else if locations::update_db(&tx, fs_location)? {
                log::debug!("{} ({}) updated.", fs_location.name, fs_location.alias);
            }
        }
        commit_tx!(tx, "failed to commit tx to add/update locations")?;

        // refresh the history
        for location in &fs_locations {
            history::reload(&mut conn, &self.weather_dir, &location.alias)?;
        }
        Ok(fs_locations.len())
    }

    /// Initialize the US cities database.
    ///
    fn us_cities_init(&self) -> crate::Result<()> {
        UsCitiesAdmin::new(&self.weather_dir).init_schema()
    }

    /// Delete the US Cities database.
    ///
    fn us_cities_delete(&self) -> crate::Result<()> {
        UsCitiesAdmin::new(&self.weather_dir).delete()
    }

    /// Load the US cities database.
    ///
    /// # Arguments
    ///
    /// * `uscities_path` contains the US cities metadata that will be loaded.
    ///
    fn us_cities_load(&self, uscities_path: &str) -> crate::Result<usize> {
        let db_admin = UsCitiesAdmin::new(&self.weather_dir);
        let count = db_admin.load_db(uscities_path)?;
        // a quick check to make sure the db is intact
        db_admin.db_details()?;
        Ok(count)
    }

    /// Retrieve information about the US Cities database.
    ///
    fn us_cities_details(&self) -> crate::Result<UsCityDetails> {
        UsCitiesAdmin::new(&self.weather_dir).db_details()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{db::sqlite::DB_FILENAME, testlib};
    use std::path::PathBuf;

    #[test]
    fn history_admin() {
        let fixture = testlib::TestFixture::create();
        let test_files = testlib::test_resources().join("db");

        // copy the locations document and 3 locations weather history
        fixture.copy_resources(&test_files);

        // make sure the environment is clean
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let db_file = PathBuf::from(&weather_dir.to_string()).join(DB_FILENAME);
        assert!(!db_file.exists());

        let testcase = SQLiteAdmin::new(Rc::new(weather_dir));
        testcase.history_drop(true).unwrap();
        testcase.history_init(false).unwrap();
        assert!(db_file.exists());
        testcase.history_load(3).unwrap();
        testcase.history_drop(false).unwrap();
        assert!(db_file.exists());
        testcase.history_drop(true).unwrap();
        assert!(!db_file.exists());
    }
}
