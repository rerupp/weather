//! The history database persistence module.
//!

use crate::backend::db::sqlite::{
    execute_sql, prepare_cached_sql, prepare_sql,
    tables::{
        named_param,
        weather::{DatesTbl, HistoryTbl, LocationsTbl, MetadataTbl},
        TblSqlBuilder,
    },
};
use chrono::NaiveDate;
use rusqlite::Transaction;
use sql_query_builder as sql;

// pull in what the inner module tests need
#[cfg(test)]
use crate::{
    backend::{
        db::sqlite::{
            commit_tx, create_tx, tables,
            weather::{db_conn, locations},
        },
        testlib, WeatherDir,
    },
    entities::Location,
};
#[cfg(test)]
use rusqlite::Connection;
#[cfg(test)]
use std::path::PathBuf;

/// Create a specific error message for history persistence.
///
/// # Params
///
/// * `args` are passed to `format!` to create the error message.
///
macro_rules! err {
    ($($args:tt)*) => {
        Err(crate::Error(format!("History persistence {}", format!($($args)*))))
    };
}

#[cfg(test)]
fn initialize_tests() -> (i64, testlib::TestFixture) {
    // initialize the test db schema
    let fixture = testlib::TestFixture::create();
    let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
    let mut conn = db_conn!(&weather_dir).unwrap();
    tables::weather::initialize_schema(&conn).unwrap();

    // add a test location
    let location = Location {
        country_name: "United States".to_string(),
        country_code: "US".to_string(),
        region_name: "Oregon".to_string(),
        region_code: "OR".to_string(),
        city_name: "Test City".to_string(),
        alias: "test".to_string(),
        latitude: "1".to_string(),
        longitude: "1".to_string(),
        tz: "UTC".to_string(),
    };
    locations::add(&mut conn, location, &weather_dir).unwrap();
    let lid = conn.last_insert_rowid();
    (lid, fixture)
}

/// Create the query that finds all dates for a location that will be used to delete data.
///
/// # Arguments
///
/// * `l` is the location column alias.
/// * `d` is the dates column alias.
///
fn location_dates_sql(l: &str, d: &str) -> String {
    sql::Select::new()
        .select(&DatesTbl::Id.alias_column(d))
        .from(&LocationsTbl::table_as(l))
        .left_join(&DatesTbl::alias_join_locations_as(d, l))
        .where_clause(&LocationsTbl::Id.alias_where_param(l))
        .to_string()
}

pub mod dates {
    //! Manage how data is inserted and deleted from the dates table.
    //!
    use super::*;

    /// Insert a row into the dates table.
    ///
    /// # Argument
    ///
    /// * `tx` is the transaction used when inserting the row.
    /// * `lid` is the associated locations table ROWID.
    /// * `sql` is the SQL insert statement.
    /// * `date` provides the contents of the row.
    ///
    pub fn insert(tx: &mut Transaction, lid: i64, sql: &str, date: NaiveDate) -> crate::Result<i64> {
        let mut stmt = prepare_cached_sql!(tx, sql, "failed to prepare {} insert SQL", DatesTbl::TABLE)?;
        let params = [named_param!(DatesTbl::Lid, lid), named_param!(DatesTbl::Date, date)];
        execute_sql!(stmt, &params, "failed to insert {} row", DatesTbl::TABLE)?;
        Ok(tx.last_insert_rowid())
    }

    /// Delete all history dates associated with a location.
    ///
    /// # Arguments
    ///
    /// * `tx` is the transaction that will be used.
    /// * `lid` is the locations ROWID.
    ///
    pub fn delete(tx: &mut Transaction, lid: i64) -> crate::Result<bool> {
        let delete_sql = sql::Delete::new()
            .delete_from(DatesTbl::TABLE)
            .where_clause(&format!("{} IN ({})", DatesTbl::Id.column(), location_dates_sql("l", "d")))
            .to_string();
        let mut stmt = prepare_sql!(tx, &delete_sql, "failed preparing {} delete SQL", DatesTbl::TABLE)?;
        let params = [named_param!(LocationsTbl::Id, lid)];
        execute_sql!(stmt, &params, "error executing {} delete statement", DatesTbl::TABLE)?;
        Ok(true)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn insert_delete() {
            // initialize the test environment
            let (lid, fixture) = initialize_tests();
            let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
            let mut conn = db_conn!(&weather_dir).unwrap();

            // verify the dates row count
            fn assert_row_count(conn: &Connection, count: usize) {
                let sql = sql::Select::new().select("COUNT(*)").from(DatesTbl::TABLE).to_string();
                let mut stmt = prepare_sql!(conn, &sql, "error preparing dates count").unwrap();
                let row_count = stmt.query_one([], |row| row.get::<_, i64>(0)).unwrap() as usize;
                assert_eq!(row_count, count);
            }

            // add a couple of dates
            assert_row_count(&conn, 0);
            let mut tx = create_tx!(conn, "did not create dates insert transaction").unwrap();
            let dates = [NaiveDate::from_ymd_opt(2026, 6, 6).unwrap(), NaiveDate::from_ymd_opt(2026, 6, 7).unwrap()];
            assert_ne!(insert(&mut tx, lid, &DatesTbl::insert_sql(), dates[0]).unwrap(), 0);
            assert_ne!(insert(&mut tx, lid, &DatesTbl::insert_sql(), dates[1]).unwrap(), 0);
            commit_tx!(tx, "did not commit date insert").unwrap();
            assert_row_count(&conn, 2);

            // check the dates are available
            fn assert_date(conn: &Connection, date: NaiveDate) {
                let query_sql = sql::Select::new()
                    .select("COUNT(*)")
                    .from(DatesTbl::TABLE)
                    .where_clause(&DatesTbl::Date.where_param())
                    .to_string();
                let mut stmt = prepare_sql!(conn, &query_sql, "error preparing dates query").unwrap();
                stmt.query_one(&[named_param!(DatesTbl::Date, date)], |row| {
                    let count = row.get::<_, i64>(0).unwrap();
                    assert_eq!(count, 1);
                    Ok(())
                })
                .unwrap();
            }
            assert_date(&conn, dates[0]);
            assert_date(&conn, dates[1]);

            let mut tx = create_tx!(conn, "did not create dates delete transaction").unwrap();
            delete(&mut tx, 1).unwrap();
            commit_tx!(tx, "did not commit dates delete").unwrap();
            assert_row_count(&conn, 0);
        }
    }
}

pub mod metadata {
    //! Manage how data is inserted and deleted from the metadata table.
    //!
    use super::*;
    use crate::backend::filesys::FilesysMetadata;

    /// Insert a row into the metadata table.
    ///
    /// # Argument
    ///
    /// * `tx` is the transaction used when inserting the row.
    /// * `did` is the associated dates table ROWID.
    /// * `sql` is the SQL insert statement.
    /// * `metadata` provides the contents of the row.
    ///
    pub fn insert(tx: &mut Transaction, did: i64, sql: &str, metadata: &FilesysMetadata) -> crate::Result<i64> {
        let mut stmt = prepare_cached_sql!(tx, sql, "failed to prepare {} insert SQL", MetadataTbl::TABLE)?;
        let params = [
            named_param!(MetadataTbl::Did, did),
            named_param!(MetadataTbl::UncompressedSize, metadata.uncompressed_size as i64),
            named_param!(MetadataTbl::CompressedSize, metadata.compressed_size as i64),
            named_param!(MetadataTbl::DataSize, metadata.data_size as i64),
        ];
        execute_sql!(stmt, &params, "failed to insert {} row", MetadataTbl::TABLE)?;
        Ok(tx.last_insert_rowid())
    }

    /// Delete all metadata data associated with a location.
    ///
    /// # Arguments
    ///
    /// * `tx` is the transaction that will be used.
    /// * `lid` is the locations ROWID.
    ///
    pub fn delete(tx: &mut Transaction, lid: i64) -> crate::Result<bool> {
        let delete_sql = sql::Delete::new()
            .delete_from(MetadataTbl::TABLE)
            .where_clause(&format!("{} IN ({})", MetadataTbl::Did.column(), location_dates_sql("l", "d")))
            .to_string();
        let mut stmt = prepare_sql!(tx, &delete_sql, "failed preparing {} delete SQL", MetadataTbl::TABLE)?;
        let params = [named_param!(LocationsTbl::Id, lid)];
        execute_sql!(stmt, &params, "error executing {} delete statement", MetadataTbl::TABLE)?;
        Ok(true)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn insert_delete() {
            // initialize the test environment
            let (lid, fixture) = initialize_tests();
            let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
            let mut conn = db_conn!(&weather_dir).unwrap();

            // verify the metadata row count
            fn assert_row_count(conn: &Connection, count: usize) {
                let sql = sql::Select::new().select("COUNT(*)").from(MetadataTbl::TABLE).to_string();
                let mut stmt = prepare_sql!(conn, &sql, "error preparing metadata count").unwrap();
                let row_count = stmt.query_one([], |row| row.get::<_, i64>(0)).unwrap() as usize;
                assert_eq!(row_count, count);
            }

            // add a couple of metadata rows
            assert_row_count(&conn, 0);
            let create_metadata = |y, m, d, size| FilesysMetadata {
                alias: "".to_string(),
                date: NaiveDate::from_ymd_opt(y, m, d).unwrap(),
                uncompressed_size: size,
                compressed_size: size + 1,
                data_size: size + 2,
            };
            let metadata_collection = vec![create_metadata(2026, 6, 7, 1), create_metadata(2026, 6, 8, 2)];
            let dates_sql = DatesTbl::insert_sql();
            let metadata_sql = MetadataTbl::insert_sql();
            let mut tx = create_tx!(conn, "did not create metadata insert transaction").unwrap();
            for metadata in &metadata_collection {
                let did = dates::insert(&mut tx, lid, &dates_sql, metadata.date).unwrap();
                assert_ne!(insert(&mut tx, did, &metadata_sql, &metadata).unwrap(), 0);
            }
            commit_tx!(tx, "did not commit metadata insert").unwrap();
            assert_row_count(&conn, 2);

            // check the metadata content
            fn assert_metadata(conn: &Connection, metadata: &FilesysMetadata) {
                let d = "d";
                let m = "m";
                let query_sql = sql::Select::new()
                    .select(&MetadataTbl::UncompressedSize.alias_column(m))
                    .select(&MetadataTbl::CompressedSize.alias_column(m))
                    .select(&MetadataTbl::DataSize.alias_column(m))
                    .from(&DatesTbl::table_as(d))
                    .left_join(&MetadataTbl::alias_join_dates(m, d))
                    .where_clause(&DatesTbl::Date.alias_where_param(d))
                    .to_string();
                let mut stmt = prepare_sql!(conn, &query_sql, "error preparing date query").unwrap();
                stmt.query_one(&[named_param!(DatesTbl::Date, metadata.date)], |row| {
                    assert_eq!(row.get::<_, i64>(0).unwrap(), metadata.uncompressed_size as i64);
                    assert_eq!(row.get::<_, i64>(1).unwrap(), metadata.compressed_size as i64);
                    assert_eq!(row.get::<_, i64>(2).unwrap(), metadata.data_size as i64);
                    Ok(())
                })
                .unwrap();
            }
            assert_metadata(&conn, &metadata_collection[0]);
            assert_metadata(&conn, &metadata_collection[1]);

            let mut tx = create_tx!(conn, "did not create metadata delete transaction").unwrap();
            delete(&mut tx, lid).unwrap();
            commit_tx!(tx, "did not commit metadata delete").unwrap();
            assert_row_count(&conn, 0);
        }
    }
}

pub mod history {
    //! Manage how data is inserted and deleted from the dates table.
    //!
    use super::*;
    use crate::entities::History;

    /// Insert a row into the history table.
    ///
    /// # Argument
    ///
    /// * `tx` is the transaction used when inserting the row.
    /// * `did` is the associated dates table id.
    /// * `sql` is the SQL insert statement.
    /// * `history` provides the contents of the row.
    ///
    pub fn insert(tx: &mut Transaction, did: i64, sql: &str, history: &History) -> crate::Result<i64> {
        let mut stmt = prepare_cached_sql!(tx, sql, "failed to prepare {} insert SQL", HistoryTbl::TABLE)?;
        let params = [
            named_param!(HistoryTbl::Did, did),
            named_param!(HistoryTbl::TempHigh, history.temperature_high),
            named_param!(HistoryTbl::TempLow, history.temperature_low),
            named_param!(HistoryTbl::TempMean, history.temperature_mean),
            named_param!(HistoryTbl::DewPoint, history.dew_point),
            named_param!(HistoryTbl::Humidity, history.humidity),
            named_param!(HistoryTbl::PrecipProb, history.precipitation_chance),
            named_param!(HistoryTbl::PrecipType, history.precipitation_type),
            named_param!(HistoryTbl::Precip, history.precipitation_amount),
            named_param!(HistoryTbl::WindSpeed, history.wind_speed),
            named_param!(HistoryTbl::WindGust, history.wind_gust),
            named_param!(HistoryTbl::WindDir, history.wind_direction),
            named_param!(HistoryTbl::CloudCover, history.cloud_cover),
            named_param!(HistoryTbl::Pressure, history.pressure),
            named_param!(HistoryTbl::UvIndex, history.uv_index),
            named_param!(HistoryTbl::Sunrise, history.sunrise),
            named_param!(HistoryTbl::Sunset, history.sunset),
            named_param!(HistoryTbl::MoonPhase, history.moon_phase),
            named_param!(HistoryTbl::Visibility, history.visibility),
            named_param!(HistoryTbl::Description, history.description),
        ];
        execute_sql!(stmt, &params, "failed to insert {} row", HistoryTbl::TABLE)?;
        Ok(tx.last_insert_rowid())
    }

    /// Delete all history data associated with a location.
    ///
    /// # Arguments
    ///
    /// * `tx` is the transaction that will be used.
    /// * `lid` is the locations ROWID.
    ///
    pub fn delete(tx: &mut Transaction, lid: i64) -> crate::Result<bool> {
        let delete_sql = sql::Delete::new()
            .delete_from(HistoryTbl::TABLE)
            .where_clause(&format!("{} IN ({})", HistoryTbl::Did.column(), location_dates_sql("l", "d")))
            .to_string();
        let mut stmt = prepare_sql!(tx, &delete_sql, "failed preparing {} delete SQL", HistoryTbl::TABLE)?;
        let params = [named_param!(LocationsTbl::Id, lid)];
        execute_sql!(stmt, &params, "error executing {} delete statement", HistoryTbl::TABLE)?;
        Ok(true)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn insert_delete() {
            // initialize the test environment
            let (lid, fixture) = initialize_tests();
            let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
            let mut conn = db_conn!(&weather_dir).unwrap();

            // verify the metadata row count
            fn assert_row_count(conn: &Connection, count: usize) {
                let sql = sql::Select::new().select("COUNT(*)").from(HistoryTbl::TABLE).to_string();
                let mut stmt = prepare_sql!(conn, &sql, "error preparing history count").unwrap();
                let row_count = stmt.query_one([], |row| row.get::<_, i64>(0)).unwrap() as usize;
                assert_eq!(row_count, count);
            }

            // add a couple of history rows
            assert_row_count(&conn, 0);
            let create_history = |y, m, d| History {
                date: NaiveDate::from_ymd_opt(y, m, d).unwrap(),
                // the tables module verifies history row contents so default the remaining attributes
                ..Default::default()
            };
            let history_collection = vec![create_history(2026, 6, 7), create_history(2026, 6, 8)];
            // let metadata_collection = vec![create_metadata(2026, 6, 7, 1), create_metadata(2026, 6, 8, 2)];
            let dates_sql = DatesTbl::insert_sql();
            let history_sql = HistoryTbl::insert_sql();
            let mut tx = create_tx!(conn, "did not create history insert transaction").unwrap();
            for history in &history_collection {
                let did = dates::insert(&mut tx, lid, &dates_sql, history.date).unwrap();
                assert_ne!(insert(&mut tx, did, &history_sql, &history).unwrap(), 0);
            }
            commit_tx!(tx, "did not commit history insert").unwrap();
            assert_row_count(&conn, 2);

            // check the metadata content
            fn assert_history(conn: &Connection, date: NaiveDate) {
                // there's only 1 location so querying the history associated with a date is ok
                let d = "d";
                let h = "h";
                let query_sql = sql::Select::new()
                    .select("COUNT(*)")
                    .from(&DatesTbl::table_as(d))
                    .inner_join(&HistoryTbl::alias_join_dates_as(h, d))
                    .where_clause(&DatesTbl::Date.alias_where_param(d))
                    .to_string();
                let mut stmt = prepare_sql!(conn, &query_sql, "error preparing date query").unwrap();
                stmt.query_one(&[named_param!(DatesTbl::Date, date)], |row| {
                    // the tables module verifies insert contents so checking the row is good enough
                    assert_eq!(row.get::<_, i64>(0).unwrap(), 1);
                    Ok(())
                })
                .unwrap();
            }
            assert_history(&conn, history_collection[0].date);
            assert_history(&conn, history_collection[1].date);

            let mut tx = create_tx!(conn, "did not create history delete transaction").unwrap();
            delete(&mut tx, lid).unwrap();
            commit_tx!(tx, "did not commit history delete").unwrap();
            assert_row_count(&conn, 0);
        }
    }
}
