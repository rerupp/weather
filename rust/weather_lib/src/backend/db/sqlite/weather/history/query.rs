//! The common weather database queries.
//!

use super::locations;
use crate::prelude::DateRange;
use crate::{
    backend::{
        db::{
            sqlite::{
                prepare_sql, query_rows,
                tables::{
                    named_param,
                    weather::{DatesTbl, HistoryTbl, LocationsTbl, MetadataTbl},
                    TblSqlBuilder,
                },
            },
            DbMetadata,
        },
        WeatherDir,
    },
    entities::{DatabaseHistorySummary, DateRanges, FilesysHistorySummary, History, HistoryDates, LocationFilter},
};
use chrono::NaiveDate;
use rusqlite::Connection;
use sql_query_builder as sql;
use std::collections::HashMap;

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(format!("Query {}", format!($($arg)*)))
    };
}

/// Get the daily weather data history for a location.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `alias` identifies the location.
/// * `date_range` selects the history dates.
///
pub fn get_history(conn: &Connection, alias: &str, date_range: DateRange) -> crate::Result<Vec<History>> {
    // create the query to get history
    let l = "l";
    let d = "d";
    let h = "h";
    let start_param = ":start";
    let end_param = ":end";
    let query = sql::Select::new()
        .select(&LocationsTbl::Id.alias_column_as_column(l))
        .select(&DatesTbl::Date.alias_column_as_column(d))
        .select(&HistoryTbl::TempHigh.alias_column_as_column(h))
        .select(&HistoryTbl::TempLow.alias_column_as_column(h))
        .select(&HistoryTbl::TempMean.alias_column_as_column(h))
        .select(&HistoryTbl::DewPoint.alias_column_as_column(h))
        .select(&HistoryTbl::Humidity.alias_column_as_column(h))
        .select(&HistoryTbl::Sunrise.alias_column_as_column(h))
        .select(&HistoryTbl::Sunset.alias_column_as_column(h))
        .select(&HistoryTbl::CloudCover.alias_column_as_column(h))
        .select(&HistoryTbl::MoonPhase.alias_column_as_column(h))
        .select(&HistoryTbl::UvIndex.alias_column_as_column(h))
        .select(&HistoryTbl::WindSpeed.alias_column_as_column(h))
        .select(&HistoryTbl::WindGust.alias_column_as_column(h))
        .select(&HistoryTbl::WindDir.alias_column_as_column(h))
        .select(&HistoryTbl::Visibility.alias_column_as_column(h))
        .select(&HistoryTbl::Pressure.alias_column_as_column(h))
        .select(&HistoryTbl::Precip.alias_column_as_column(h))
        .select(&HistoryTbl::PrecipProb.alias_column_as_column(h))
        .select(&HistoryTbl::PrecipType.alias_column_as_column(h))
        .select(&HistoryTbl::Description.alias_column_as_column(h))
        .from(&LocationsTbl::table_as(l))
        .inner_join(&DatesTbl::alias_join_locations_as(d, l))
        .inner_join(&HistoryTbl::alias_join_dates_as(h, d))
        .where_and(&LocationsTbl::Alias.alias_where_param(l))
        .where_and(&format!("{} BETWEEN {} AND {}", DatesTbl::Date.alias_column(d), start_param, end_param))
        .order_by(&DatesTbl::Date.alias_column(d))
        .to_string();
    //     let alias = location.alias.as_str();
    let stmt = prepare_sql!(conn, &query, "failed to prepare history query")?;
    let params = [
        named_param!(LocationsTbl::Alias, alias),
        (start_param, &date_range.start),
        (end_param, &date_range.end),
    ];
    let mut histories = vec![];
    query_rows(stmt, &params, |row| {
        let history = History {
            alias: alias.to_string(),
            date: row.get(DatesTbl::Date.column()).unwrap(),
            temperature_high: row.get(HistoryTbl::TempHigh.column()).unwrap(),
            temperature_low: row.get(HistoryTbl::TempLow.column()).unwrap(),
            temperature_mean: row.get(HistoryTbl::TempMean.column()).unwrap(),
            dew_point: row.get(HistoryTbl::DewPoint.column()).unwrap(),
            humidity: row.get(HistoryTbl::Humidity.column()).unwrap(),
            precipitation_chance: row.get(HistoryTbl::PrecipProb.column()).unwrap(),
            precipitation_type: row.get(HistoryTbl::PrecipType.column()).unwrap(),
            precipitation_amount: row.get(HistoryTbl::Precip.column()).unwrap(),
            wind_speed: row.get(HistoryTbl::WindSpeed.column()).unwrap(),
            wind_gust: row.get(HistoryTbl::WindGust.column()).unwrap(),
            wind_direction: row.get(HistoryTbl::WindDir.column()).unwrap(),
            cloud_cover: row.get(HistoryTbl::CloudCover.column()).unwrap(),
            pressure: row.get(HistoryTbl::Pressure.column()).unwrap(),
            uv_index: row.get(HistoryTbl::UvIndex.column()).unwrap(),
            sunrise: row.get(HistoryTbl::Sunrise.column()).unwrap(),
            sunset: row.get(HistoryTbl::Sunset.column()).unwrap(),
            moon_phase: row.get(HistoryTbl::MoonPhase.column()).unwrap(),
            visibility: row.get(HistoryTbl::Visibility.column()).unwrap(),
            description: row.get(HistoryTbl::Description.column()).unwrap(),
        };
        histories.push(history);
        Ok(())
    })?;
    Ok(histories)
}

/// Get the location history dates.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `filters_opt` are the optional location filters.
///
pub fn history_dates(conn: &Connection, filters_opt: Option<Vec<LocationFilter>>) -> crate::Result<Vec<HistoryDates>> {
    // generate the history dates query
    let l = "l";
    let d = "d";
    let mut query = sql::Select::new()
        .select(&LocationsTbl::Alias.alias_column_as_column(l))
        .select(&DatesTbl::Date.alias_column_as_column(d))
        .from(&LocationsTbl::table_as(l))
        .inner_join(&DatesTbl::alias_join_locations_as(d, l));
    if let Some(filters) = &filters_opt {
        query = locations::add_location_filters(query, filters, Some(l));
    }
    let sql =
        query.order_by(&LocationsTbl::Alias.alias_column(l)).order_by(&DatesTbl::Date.alias_column(d)).to_string();

    // execute the query
    let stmt = prepare_sql!(conn, &sql, "failed to prepare history dates query")?;
    let mut location_dates: Vec<(String, Vec<NaiveDate>)> = vec![];
    query_rows(stmt, [], |row| {
        let alias = row.get::<_, String>(LocationsTbl::Alias.column()).unwrap();
        let date = row.get::<_, NaiveDate>(DatesTbl::Date.column()).unwrap();
        match location_dates.last_mut() {
            None => location_dates.push((alias, vec![date])),
            Some((last_alias, dates)) => match last_alias.as_str() == alias.as_str() {
                true => dates.push(date),
                false => location_dates.push((alias, vec![date])),
            },
        }
        Ok(())
    })?;

    // create the history dates collection
    let mut dates_map = location_dates.into_iter().collect::<HashMap<String, Vec<NaiveDate>>>();
    let locations = locations::get(conn, filters_opt)?;
    let history_dates = locations
        .into_iter()
        .map(|location| {
            let dates = dates_map.remove(&location.alias).unwrap_or(vec![]);
            let date_ranges = DateRanges::new(&location.alias, dates).date_ranges;
            HistoryDates { location, history_dates: date_ranges }
        })
        .collect::<Vec<_>>();

    Ok(history_dates)
}

/// For each location calculate the amount of space used in the database to store history.
/// The returned collection contains the location alias with history count and database space used.
///
/// This is terribly expensive but it suffices for right now
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn db_size(conn: &Connection) -> crate::Result<HashMap<String, (usize, DatabaseHistorySummary)>> {
    // get the count of history dates for each location
    let mut history_counts = history_counts(conn)?;
    let total_histories = history_counts.iter().map(|(_, count)| *count).sum::<usize>() as f64;

    // get the overall size of history in the database
    let dates_md = db_metadata(conn, DatesTbl::TABLE)?;
    let metadata_md = db_metadata(conn, MetadataTbl::TABLE)?;
    let history_md = db_metadata(conn, HistoryTbl::TABLE)?;
    let total_data_size = (dates_md.data_size + metadata_md.data_size + history_md.data_size) as f64;
    let total_index_size = (dates_md.index_size + metadata_md.index_size + history_md.index_size) as f64;
    let total_data_unused = (dates_md.data_unused + metadata_md.data_unused + history_md.data_unused) as f64;
    let total_index_unused = (dates_md.index_unused + metadata_md.index_unused + history_md.index_unused) as f64;

    // calculate the sizes based on the number of histories
    let mut db_sizes: HashMap<String, (usize, DatabaseHistorySummary)> = HashMap::new();
    history_counts.drain().for_each(|(alias, count)| {
        let percentage = count as f64 / total_histories;
        let summary = DatabaseHistorySummary {
            data_bytes: (total_data_size * percentage).round() as u64,
            unused_data_bytes: (total_data_unused * percentage).round() as u64,
            index_bytes: (total_index_size * percentage).round() as u64,
            unused_index_bytes: (total_index_unused * percentage).round() as u64,
        };
        db_sizes.insert(alias, (count, summary));
    });
    Ok(db_sizes)
}

/// For each location get the filesystem related metadata.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather history data directory.
///
pub fn fs_size(conn: &Connection, weather_dir: &WeatherDir) -> crate::Result<HashMap<String, FilesysHistorySummary>> {
    let l = "l";
    let d = "d";
    let m = "m";
    let select = sql::Select::new()
        .select(&LocationsTbl::Alias.alias_column_as_column(l))
        .select(&MetadataTbl::UncompressedSize.alias_sum_as_column(m))
        .select(&MetadataTbl::CompressedSize.alias_sum_as_column(m))
        .select(&MetadataTbl::DataSize.alias_sum_as_column(m))
        .from(&LocationsTbl::table_as(l))
        .left_join(&DatesTbl::alias_join_locations_as(d, l))
        .left_join(&MetadataTbl::alias_join_dates(m, d))
        .group_by(&LocationsTbl::Alias.alias_column(l))
        .order_by(&LocationsTbl::Alias.alias_column(l))
        .to_string();
    let stmt = prepare_sql!(conn, &select, "failed to prepare metadata size query")?;
    let mut alias_metadata: HashMap<String, FilesysHistorySummary> = HashMap::new();
    query_rows(stmt, [], |row| {
        let alias = row.get::<_, String>(LocationsTbl::Alias.column()).unwrap();
        let summary = FilesysHistorySummary {
            uncompressed_size: row.get::<_, i64>(MetadataTbl::UncompressedSize.column()).unwrap_or(0) as u64,
            compressed_size: row.get::<_, i64>(MetadataTbl::CompressedSize.column()).unwrap_or(0) as u64,
            data_size: row.get::<_, i64>(MetadataTbl::DataSize.column()).unwrap_or(0) as u64,
            archive_size: weather_dir.archive(&alias).size(),
        };
        alias_metadata.insert(alias, summary);
        Ok(())
    })?;
    Ok(alias_metadata)
}

/// Used internally to calculate the data space used by a table.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `table` is the database table whose space will be calculated.
///
fn db_metadata(conn: &Connection, table: &str) -> crate::Result<DbMetadata> {
    // get the index names associated with the table
    let index_query = sql::Select::new().select("name").from(&format!("pragma_index_list('{table}')")).to_string();
    let stmt = prepare_sql!(conn, &index_query, "failed to prepare index query")?;
    let mut index_names: Vec<String> = vec![];
    query_rows(stmt, [], |row| {
        let name = row.get::<_, String>("name").unwrap();
        index_names.push(name);
        Ok(())
    })?;

    // get the bytes sizes used by the table
    let name = "name";
    let pgsize = "pgsize";
    let unused = "unused";
    let size_query = sql::Select::new()
        .select(name)
        .select(&format!("SUM({pgsize}) AS {pgsize}"))
        .select(&format!("SUM({unused}) as {unused}"))
        .from("dbstat")
        .where_clause(&format!("{name} LIKE '%{table}%'"))
        .group_by(name)
        .to_string();
    let stmt = prepare_sql!(conn, &size_query, "failed to prepare table size query")?;
    let mut metadata = DbMetadata { table: table.to_string(), ..Default::default() };
    query_rows(stmt, [], |row| {
        let name = row.get::<_, String>(name).unwrap();
        let size = row.get::<_, i64>(pgsize).unwrap() as usize;
        let unused = row.get::<_, i64>(unused).unwrap() as usize;
        match index_names.contains(&name) {
            true => {
                metadata.index_size += size;
                metadata.index_unused += unused;
            }
            false => {
                metadata.data_size += size;
                metadata.data_unused += unused;
            }
        }
        Ok(())
    })?;
    Ok(metadata)
}

/// Used internally to help calculate the amount of history space being used by locations.
///
/// # Arguments
///
/// * `conn` is the connection that will be used.
/// * `table_name` is the table name that will be examined.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn history_counts(conn: &Connection) -> crate::Result<HashMap<String, usize>> {
    // query the history counts
    let l = "l";
    let d = "d";
    let count = "count";
    let query = sql::Select::new()
        .select(&LocationsTbl::Alias.alias_column_as_column(l))
        .select(&DatesTbl::Date.alias_count_as(d, count))
        .from(&LocationsTbl::table_as(l))
        .left_join(&DatesTbl::alias_join_locations_as(d, l))
        .group_by(&LocationsTbl::Alias.alias_column(l))
        .order_by(&LocationsTbl::Alias.alias_column(l))
        .to_string();
    let stmt = prepare_sql!(conn, &query, "failed to prepare history counts query")?;

    let mut history_counts: HashMap<String, usize> = HashMap::new();
    query_rows(stmt, [], |row| {
        let alias = row.get::<_, String>(LocationsTbl::Alias.column()).unwrap();
        let count = row.get::<_, i64>(count).unwrap() as usize;
        history_counts.insert(alias, count);
        Ok(())
    })?;
    Ok(history_counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend::{
            db::{
                admin::{create_db_admin, DbAdmin},
                sqlite::weather::{self, locations},
            },
            testlib::{self, TestFixture},
            WeatherDir,
        },
        prelude::{DateRange, Location, LocationFilter},
    };
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn init() -> TestFixture {
        let fixture = TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("db"));
        let fixture_path = PathBuf::from(&fixture);

        // initialize the database
        let db_admin = Box::new(create_db_admin(WeatherDir::new(fixture_path.clone()).unwrap()));
        db_admin.history_init(false).unwrap();
        db_admin.history_load(3).unwrap();
        fixture
    }

    #[test]
    fn fs_size() {
        let fixture = init();
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
        let mut conn = weather::db_conn!(weather_dir).unwrap();

        // add a location without any history
        let empty_location = Location {
            country_name: "United States".to_string(),
            country_code: "US".to_string(),
            region_name: "Oregon".to_string(),
            region_code: "OR".to_string(),
            city_name: "Test City".to_string(),
            alias: "test".to_string(),
            latitude: "1".to_string(),
            longitude: "-1".to_string(),
            tz: "UTC".to_string(),
        };
        locations::add(&mut conn, empty_location, &weather_dir).unwrap();

        let alias_metadata = super::fs_size(&conn, &weather_dir).unwrap();

        let testcase = alias_metadata.get("between").unwrap();
        assert_eq!(testcase.uncompressed_size, 10047);
        assert_eq!(testcase.compressed_size, 6497);
        assert_eq!(testcase.data_size, 8208);
        assert_eq!(testcase.archive_size, 10405);

        let testcase = alias_metadata.get("north").unwrap();
        assert_eq!(testcase.uncompressed_size, 10056);
        assert_eq!(testcase.data_size, 8110);
        assert_eq!(testcase.compressed_size, 6515);
        assert_eq!(testcase.archive_size, 10191);

        let testcase = alias_metadata.get("south").unwrap();
        assert_eq!(testcase.uncompressed_size, 9749);
        assert_eq!(testcase.data_size, 7896);
        assert_eq!(testcase.compressed_size, 6301);
        assert_eq!(testcase.archive_size, 9977);

        let testcase = alias_metadata.get("test").unwrap();
        assert_eq!(testcase.uncompressed_size, 0);
        assert_eq!(testcase.data_size, 0);
        assert_eq!(testcase.compressed_size, 0);
        assert_eq!(testcase.archive_size, 22);
    }

    #[test]
    fn history_dates() {
        // set up the test environment
        let fixture = init();
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
        let conn = weather::db_conn!(weather_dir).unwrap();

        // query the history dates
        let history_dates = super::history_dates(&conn, None).unwrap();
        let mut testcases = history_dates.into_iter();

        let date = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();

        // verify the results
        let testcase = testcases.next().unwrap();
        assert_eq!(testcase.location.alias, "between");
        // north and south have the same dates
        let north_south_dates = [
            DateRange::new(date(2015, 4, 1), date(2015, 4, 14)),
            DateRange::new(date(2016, 10, 10), date(2016, 10, 17)),
            DateRange::new(date(2017, 7, 14), date(2017, 7, 20)),
        ];
        assert_eq!(testcase.history_dates.len(), north_south_dates.len());
        testcase.history_dates.iter().zip(&north_south_dates).for_each(|(lhs, rhs)| assert_eq!(lhs, rhs));

        let testcase = testcases.next().unwrap();
        assert_eq!(testcase.location.alias, "north");
        assert_eq!(testcase.history_dates.len(), north_south_dates.len());
        testcase.history_dates.iter().zip(&north_south_dates).for_each(|(lhs, rhs)| assert_eq!(lhs, rhs));

        let testcase = testcases.next().unwrap();
        assert_eq!(testcase.location.alias, "south");
        let south_dates = [
            DateRange::new(date(2015, 4, 1), date(2015, 4, 14)),
            DateRange::new(date(2016, 10, 10), date(2016, 10, 17)),
            DateRange::new(date(2018, 1, 1), date(2018, 1, 7)),
        ];
        assert_eq!(testcase.history_dates.len(), south_dates.len());
        testcase.history_dates.iter().zip(&south_dates).for_each(|(lhs, rhs)| assert_eq!(lhs, rhs));

        assert!(testcases.next().is_none());

        // include filters
        let filters = vec![LocationFilter::alias("south"), LocationFilter::alias("north")];
        let history_dates = super::history_dates(&conn, Some(filters)).unwrap();
        let mut testcases = history_dates.into_iter();

        let testcase = testcases.next().unwrap();
        assert_eq!(testcase.location.alias, "north");
        assert_eq!(testcase.history_dates.len(), north_south_dates.len());
        testcase.history_dates.iter().zip(&north_south_dates).for_each(|(lhs, rhs)| assert_eq!(lhs, rhs));

        let testcase = testcases.next().unwrap();
        assert_eq!(testcase.location.alias, "south");
        assert_eq!(testcase.history_dates.len(), south_dates.len());
        testcase.history_dates.iter().zip(&south_dates).for_each(|(lhs, rhs)| assert_eq!(lhs, rhs));
    }

    #[test]
    fn db_metadata() {
        // set up the test environment
        let fixture = init();
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
        let conn = weather::db_conn!(weather_dir).unwrap();

        let testcase = super::db_metadata(&conn, LocationsTbl::TABLE).unwrap();
        assert_eq!(testcase.table, LocationsTbl::TABLE);
        assert_ne!(testcase.data_size, 0);
        assert_ne!(testcase.data_unused, 0);
        assert_ne!(testcase.index_size, 0);
        assert_ne!(testcase.index_unused, 0);

        let testcase = super::db_metadata(&conn, MetadataTbl::TABLE).unwrap();
        assert_eq!(testcase.table, MetadataTbl::TABLE);
        assert_ne!(testcase.data_size, 0);
        assert_ne!(testcase.data_unused, 0);
        assert_ne!(testcase.index_size, 0);
        assert_ne!(testcase.index_unused, 0);

        let testcase = super::db_metadata(&conn, HistoryTbl::TABLE).unwrap();
        assert_eq!(testcase.table, HistoryTbl::TABLE);
        assert_ne!(testcase.data_size, 0);
        assert_ne!(testcase.data_unused, 0);
        assert_ne!(testcase.index_size, 0);
        assert_ne!(testcase.index_unused, 0);
    }

    #[test]
    fn history_counts() {
        // set up the test environment
        let fixture = init();
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
        let conn = weather::db_conn!(weather_dir).unwrap();

        let history_counts = super::history_counts(&conn).unwrap();
        assert_eq!(history_counts["between"], 29);
        assert_eq!(history_counts["north"], 29);
        assert_eq!(history_counts["south"], 29);
    }

    #[test]
    fn get_history() {
        let fixture = init();
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
        let conn = weather::db_conn!(weather_dir).unwrap();

        macro_rules! date {
            ($y: literal, $m: literal, $d: literal) => {
                NaiveDate::from_ymd_opt($y, $m, $d).unwrap()
            };
        }

        // get some testdata history
        let alias = "between";
        let date_range = DateRange::new(date!(2015, 4, 1), date!(2015, 4, 4));
        let histories = super::get_history(&conn, alias, date_range.clone()).unwrap();
        assert_eq!(histories.len(), 4);

        // this is so fragile but verify the content of some row
        let history = histories.first().unwrap();
        // println!("{history:#?}");
        assert_eq!(history.alias, alias);
        assert_eq!(history.date, date!(2015, 4, 1));
        assert_eq!(history.temperature_high, Some(52.43));
        assert_eq!(history.temperature_low, Some(32.78));
        assert_eq!(history.temperature_mean, Some(41.99));
        assert_eq!(history.dew_point, Some(18.58));
        assert_eq!(history.humidity, Some(0.41));
        assert_eq!(history.precipitation_chance, Some(0.02));
        assert_eq!(history.precipitation_type, None);
        assert_eq!(history.precipitation_amount, Some(0.0));
        assert_eq!(history.wind_speed, Some(5.56));
        assert_eq!(history.wind_gust, Some(15.74));
        assert_eq!(history.wind_direction, Some(335));
        assert_eq!(history.cloud_cover, Some(0.13));
        assert_eq!(history.pressure, Some(1018.0));
        assert_eq!(history.uv_index, Some(8.0));
        assert_eq!(history.sunrise, date!(2015, 4, 1).and_hms_opt(13, 44, 0));
        assert_eq!(history.sunset, date!(2015, 4, 2).and_hms_opt(2, 23, 0));
        assert_eq!(history.moon_phase, Some(0.43));
        assert_eq!(history.visibility, Some(9.819));
        assert_eq!(history.description, Some("Clear throughout the day.".into()));
    }
}
