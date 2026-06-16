//! The implementation of [DbAdmin] for Sqlite3 databases.
//! .
mod history_loader;

use super::{
    cities, commit_tx, create_tx, prepare_sql, query_rows,
    tables::{
        self,
        weather::{DatesTbl, LocationsTbl, MetadataTbl},
        TblSqlBuilder,
    },
    weather,
};
use crate::{
    admin_prelude::{CitiesDetails, DbDetails, DbProblems, LocationDetails},
    backend::{
        db::admin::DbAdmin,
        filesys::{fs_lib, WeatherDir},
    },
    entities::LocationFilter,
};
use sql_query_builder as sql;
use std::fmt::Formatter;

/// Create an error from history specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error(format!("SQLite admin {}", format!($($arg)*))))
    };
}

pub(in crate::backend::db) struct SQLiteAdmin {
    weather_dir: WeatherDir,
}
impl std::fmt::Debug for SQLiteAdmin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SQLiteAdmin({})", self.weather_dir)
    }
}
impl SQLiteAdmin {
    pub fn new(weather_dir: WeatherDir) -> Self {
        Self { weather_dir }
    }
}
impl DbAdmin for SQLiteAdmin {
    /// Initialize the weather history database schema.
    ///
    /// # Arguments
    ///
    /// * `is_update` when true will initialize the history schema regardless if it exists.
    ///
    fn history_init(&self, is_update: bool) -> crate::Result<bool> {
        log::debug!("initializing history schema");
        let conn = weather::db_conn!(&self.weather_dir)?;

        match is_update || !tables::weather::is_schema_initialized(&conn)? {
            false => Ok(false),
            true => {
                tables::weather::initialize_schema(&conn)?;
                Ok(true)
            }
        }
    }

    /// Deletes the current database schema.
    ///
    /// # Arguments
    ///
    /// * `delete` when true will remove the database file.
    ///
    fn history_drop(&self, delete: bool) -> crate::Result<()> {
        if weather::db_exists(&self.weather_dir) {
            match delete {
                true => {
                    weather::db_delete(&self.weather_dir);
                }
                false => {
                    log::debug!("dropping history schema");
                    let conn = weather::db_conn!(&self.weather_dir)?;
                    tables::weather::drop_schema(&conn)?;
                    // recover any unused space
                    if let Err(error) = conn.execute("VACUUM", ()) {
                        err!("failed to recover unused space in history database: {:?}", error)?;
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
        let mut conn = weather::db_conn!(&self.weather_dir)?;
        weather::locations::load(&mut conn, &self.weather_dir)?;
        crate::log_elapsed_time!("history_loader");
        history_loader::load(conn, &self.weather_dir, threads)
    }

    /// Mine information about the weather history database.
    ///
    fn history_details(&self) -> crate::Result<Option<DbDetails>> {
        let mut db_details = None;
        if weather::db_exists(&self.weather_dir) {
            // query the db details
            let conn = weather::db_conn!(&self.weather_dir)?;

            // create the query
            let l = "l";
            let d = "d";
            let m = "m";
            let days = "days";
            let size = "size";
            let query = sql::Select::new()
                .select(&LocationsTbl::Alias.alias_column_as_column(l))
                .select(&MetadataTbl::DataSize.alias_sum_as(m, size))
                .select(&DatesTbl::Date.alias_count_as(d, days))
                .from(&LocationsTbl::table_as(l))
                .left_join(&DatesTbl::alias_join_locations_as(d, l))
                .left_join(&MetadataTbl::alias_join_dates(m, d))
                .group_by(&LocationsTbl::Alias.alias_column(l))
                .order_by(&LocationsTbl::Alias.alias_column(l))
                .to_string();
            let stmt = prepare_sql!(conn, &query, "failed to prepare db details query")?;

            // get the results
            let mut location_details = vec![];
            query_rows(stmt, [], |row| {
                location_details.push(LocationDetails {
                    alias: row.get("alias").unwrap(),
                    // if there are no histories the size will not be available in the row
                    size: row.get::<_, i64>(size).unwrap_or(0) as usize,
                    histories: row.get::<_, i64>(days).unwrap() as usize,
                });
                Ok(())
            })?;
            db_details.replace(DbDetails { size: weather::db_size(&self.weather_dir) as usize, location_details });
        }
        Ok(db_details)
    }

    /// Scans weather data history making sure the database is in sync with the backing store.
    ///
    /// # Arguments
    ///
    /// * `repair` is currently not implemented.
    ///
    // #[allow(unused_variables)]
    fn history_check(&self, _repair: bool) -> Option<DbProblems> {
        let mut conn = match weather::db_conn!(&self.weather_dir) {
            Ok(conn) => conn,
            Err(error) => {
                return Some(DbProblems::from(error));
            }
        };
        let location_problems = match weather::locations::check(&mut conn, &self.weather_dir) {
            Ok(location_problems) => location_problems,
            Err(error) => {
                return Some(DbProblems::from(error));
            }
        };
        let history_problems = match weather::history::check(&mut conn, &self.weather_dir) {
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
    /// * `filters` identifies the locations that will be reloaded.
    ///
    fn history_reload(&self, filters: Vec<LocationFilter>) -> crate::Result<usize> {
        let fs_locations = fs_lib::get_locations(&self.weather_dir, Some(filters.clone()))?;
        if fs_locations.is_empty() {
            log::warn!("There are no locations in the filesystem to reload.");
            return Ok(0);
        }

        // get locations from the database
        let mut conn = weather::db_conn!(&self.weather_dir)?;
        let db_locations = weather::locations::get(&conn, Some(filters.clone()))?;

        // refresh the db locations
        let tx = create_tx!(conn, "failed to create tx to add/update locations")?;
        for fs_location in &fs_locations {
            if !db_locations.iter().any(|db_location| fs_location.alias == db_location.alias) {
                weather::locations::add_db(&tx, fs_location.clone())?;
                log::debug!("{fs_location} added.");
            } else if weather::locations::update_db(&tx, fs_location)? {
                log::debug!("{fs_location} updated.");
            }
        }
        commit_tx!(tx, "failed to commit tx to add/update locations")?;

        // refresh the history
        for location in &fs_locations {
            weather::history::reload(&mut conn, &self.weather_dir, &location.alias)?;
        }
        Ok(fs_locations.len())
    }

    /// Initialize the Cities database.
    ///
    fn cities_init(&self) -> crate::Result<()> {
        let conn = cities::db_conn!(&self.weather_dir)?;
        tables::cities::initialize_schema(&conn)?;
        Ok(())
    }

    /// Delete the Cities database.
    ///
    /// # Arguments
    ///
    /// * `is_delete` when true will delete the database file.
    ///
    fn cities_drop(&self, is_delete: bool) -> crate::Result<()> {
        match is_delete {
            true => cities::delete(&self.weather_dir),
            false => {
                let conn = cities::db_conn!(&self.weather_dir)?;
                tables::cities::drop_schema(&conn)?;
                match conn.execute("VACUUM", ()) {
                    Ok(_) => Ok(()),
                    Err(error) => err!("failed to recover unused space in history database: {error:?}"),
                }
            }
        }
    }

    /// Load a country cities metadata into the database.
    ///
    /// # Arguments
    ///
    /// * `csv_database` contains the CSV cities metadata that will be loaded.
    /// * `reload` will remove existing country cities before adding the cities.
    ///
    fn cities_load(&self, uscities_path: &str, reload: bool) -> crate::Result<usize> {
        cities::load(&self.weather_dir, uscities_path, reload)
    }

    /// Get details about the Cities database.
    ///
    fn cities_details(&self) -> crate::Result<Option<CitiesDetails>> {
        cities::details(&self.weather_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testlib;

    #[test]
    fn history_admin() {
        let fixture = testlib::TestFixture::create();
        let test_files = testlib::test_resources().join("db");

        // copy the locations document and 3 locations weather history
        fixture.copy_resources(&test_files);

        // make sure the environment is clean
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        assert!(!weather::db_exists(&weather_dir));

        let testcase = SQLiteAdmin::new(weather_dir.clone());
        testcase.history_drop(true).unwrap();
        testcase.history_init(false).unwrap();
        assert!(weather::db_exists(&weather_dir));
        testcase.history_load(3).unwrap();
        testcase.history_drop(false).unwrap();
        assert!(weather::db_exists(&weather_dir));
        testcase.history_drop(true).unwrap();
        assert!(!weather::db_exists(&weather_dir));
    }

    #[test]
    fn history_details() {
        let fixture = testlib::TestFixture::create();
        let test_files = testlib::test_resources().join("db");

        // initialize a test db
        fixture.copy_resources(&test_files);
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let admin = SQLiteAdmin::new(weather_dir);
        admin.history_init(false).unwrap();
        admin.history_load(3).unwrap();

        // add a new location
        let location = crate::entities::Location {
            country_name: "Country".to_string(),
            country_code: "CO".to_string(),
            region_name: "Region".to_string(),
            region_code: "RN".to_string(),
            city_name: "Test".to_string(),
            alias: "test".to_string(),
            latitude: "1".to_string(),
            longitude: "1".to_string(),
            tz: "UTC".to_string(),
        };

        // add a location without history
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let mut conn = super::weather::db_conn!(weather_dir).unwrap();
        weather::locations::add(&mut conn, location, &weather_dir).unwrap();

        // get the testcase
        let details_opt = admin.history_details().unwrap();
        let testcase = details_opt.unwrap();
        assert_eq!(testcase.size, weather::db_size(&weather_dir) as usize);
        assert_eq!(testcase.location_details.len(), 4);

        // these tests are highly dependent on the resource contents
        // todo: fix the size compares???
        assert_eq!(testcase.location_details[0].alias, "between");
        assert_ne!(testcase.location_details[0].size, 0);
        assert_eq!(testcase.location_details[0].histories, 29);

        assert_eq!(testcase.location_details[1].alias, "north");
        assert_ne!(testcase.location_details[1].size, 0);
        assert_eq!(testcase.location_details[1].histories, 29);

        assert_eq!(testcase.location_details[2].alias, "south");
        assert_ne!(testcase.location_details[2].size, 0);
        assert_eq!(testcase.location_details[2].histories, 29);

        assert_eq!(testcase.location_details[3].alias, "test");
        assert_eq!(testcase.location_details[3].size, 0);
        assert_eq!(testcase.location_details[3].histories, 0);
    }
}
