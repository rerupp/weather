//! This module manages weather data history in the database.

mod query;
pub use query::history_dates;

use super::{
    commit_tx, create_tx, estimate_size, execute_sql, locations, metadata, prepare_cached_sql, prepare_sql, query_rows,
    SqlResult,
};
use crate::{
    admin::entities::DbHistoryProblemDetails,
    admin_prelude::DbHistoryProblems,
    backend::filesys::{fs_lib, WeatherDir},
    entities::{DailyHistories, DateRange, History, HistorySummaries, Location, LocationFilter},
};
use rusqlite::{named_params, Connection, Row, Transaction};

/// Create a database history specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("history {}", format!($($arg)*)))
    }
}

/// Create an error from history specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}
pub fn add(
    conn: &mut Connection,
    weather_dir: &WeatherDir,
    mut daily_histories: DailyHistories,
) -> crate::Result<usize> {
    let location = &daily_histories.location;

    // make sure the database knows about the location
    let lid = locations::location_id(conn, &location.alias)?;

    // unfortunately the history archive does this when it adds
    daily_histories.histories.sort_by(|lhs, rhs| lhs.date.cmp(&rhs.date));
    daily_histories.histories.dedup_by(|lhs, rhs| lhs.date == rhs.date);

    // the history archive will make sure there are no duplicates added and issue log warnings
    let added_metadata = fs_lib::add_daily_history(weather_dir, &daily_histories)?;
    let added_histories = match added_metadata.len() {
        0 => return Ok(0),
        len => len,
    };

    // remove the histories that were not added
    daily_histories.histories.retain(|history| added_metadata.iter().any(|md| history.date == md.date));

    // make sure the metadata and histories are in sync
    #[cfg(debug_assertions)]
    for i in 1..added_metadata.len() {
        assert_eq!(daily_histories.histories[i].date, added_metadata[i].date);
    }

    // for the database update, combine the histories and metadata
    let updates = daily_histories
        .histories
        .into_iter()
        .zip(added_metadata.into_iter())
        .map(|history_metadata| history_metadata)
        .collect::<Vec<_>>();

    // add the histories
    let size = estimate_size(&conn, "history")?;
    let mut tx = create_tx!(conn, "failed to create insert transaction")?;
    for (history, md) in updates {
        let size = size
            + history.description.as_ref().map_or(0, |s| s.len())
            + history.precipitation_type.as_ref().map_or(0, |s| s.len());
        insert_history(&mut tx, lid, size, md.compressed_size as usize, &history)?;
    }
    commit_tx!(tx, "failed to commit daily histories")?;
    Ok(added_histories)
}

/// Add weather history into the database.
///
/// # Arguments
///
/// * `tx` is the transaction associate with the data insertion.
/// * 'lid' is the location database id.
/// * `size` is the size in bytes of the db history data.
/// * `store_size` is the size in bytes of the backing archive history data.
/// * `history` is the weather history that will be added.
///
pub fn insert_history(
    tx: &mut Transaction,
    lid: i64,
    size: usize,
    store_size: usize,
    history: &History,
) -> crate::Result<()> {
    let mid = metadata::insert(tx, lid, &history.date, store_size, size)?;
    const INSERT_SQL: &str = r#"
    INSERT INTO history (
        mid, temp_high, temp_low, temp_mean, dew_point, humidity, sunrise_t, sunset_t, cloud_cover, moon_phase,
        uv_index, wind_speed, wind_gust, wind_dir, visibility, pressure, precip, precip_prob, precip_type, description
    )
    VALUES (
        :mid, :temp_high, :temp_low, :temp_mean, :dew_point, :humidity, :sunrise_t, :sunset_t, :cloud_cover, :moon_phase,
        :uv_index, :wind_speed, :wind_gust, :wind_dir, :visibility, :pressure, :precip, :precip_prob, :precip_type, :description
    )"#;
    let mut stmt = prepare_cached_sql!(tx, INSERT_SQL, "failed to prepare insert history SQL")?;
    let params = named_params![
        ":mid": mid,
        ":temp_high": history.temperature_high,
        ":temp_low": history.temperature_low,
        ":temp_mean": history.temperature_mean,
        ":dew_point": history.dew_point,
        ":humidity": history.humidity,
        ":sunrise_t": history.sunrise,
        ":sunset_t": history.sunset,
        ":cloud_cover": history.cloud_cover,
        ":moon_phase": history.moon_phase,
        ":uv_index": history.uv_index,
        ":wind_speed": history.wind_speed,
        ":wind_gust": history.wind_gust,
        ":wind_dir": history.wind_direction,
        ":visibility": history.visibility,
        ":pressure": history.pressure,
        ":precip": history.precipitation_amount,
        ":precip_prob": history.precipitation_chance,
        ":precip_type": history.precipitation_type,
        ":description": history.description,
    ];
    execute_sql!(stmt, params, "failed to insert history")?;
    Ok(())
}

/// Get the daily weather data history for a location.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `location` is whose history will be returned.
/// * `date_range` is the history dates to query.
pub fn get(conn: &mut Connection, location: Location, date_range: DateRange) -> crate::Result<DailyHistories> {
    // query the
    const HISTORY_SQL: &str = r#"
        SELECT
            l.id AS lid, m.date AS date,
            h.temp_high AS temp_high, h.temp_low AS temp_low, h.temp_mean AS temp_mean,
            h.dew_point AS dew_point, h.humidity AS humidity,
            h.sunrise_t AS sunrise_t, h.sunset_t AS sunset_t,
            h.cloud_cover AS cloud_cover, h.moon_phase AS moon_phase, h.uv_index AS uv_index,
            h.wind_speed AS wind_speed, h.wind_gust AS wind_gust, h.wind_dir AS wind_dir,
            h.visibility as visibility, h.pressure as pressure,
            h.precip as precip, h.precip_prob as precip_prob, h.precip_type as precip_type,
            h.description AS description
        FROM locations AS l
            INNER JOIN metadata AS m ON l.id=m.lid
            INNER JOIN history AS h ON m.id=h.mid
        WHERE
            l.alias=:alias AND m.date BETWEEN :from AND :thru
        ORDER BY date
    "#;
    let alias = location.alias.as_str();
    let mut stmt = prepare_sql!(conn, HISTORY_SQL, "failed to prepare history query")?;
    let params = named_params![":alias": alias, ":from": date_range.start, ":thru": date_range.end];
    let mut rows = query_rows!(stmt, params, "'{}' history query failed", alias)?;
    let mut histories = vec![];
    loop {
        match rows.next() {
            Ok(None) => break,
            Err(error) => err!("failed to get next history row: {:?}", error)?,
            Ok(Some(row)) => match row_to_history(alias, row) {
                Ok(history) => histories.push(history),
                Err(error) => err!("failed to create history from row: {:?}", error)?,
            },
        }
    }
    Ok(DailyHistories { location, histories })
}

/// Create history from the database.
///
/// # Arguments
///
/// * `alias` is the location alias name.
/// * `row` the query row that will be converted into History.
///
fn row_to_history(alias: &str, row: &Row) -> SqlResult<History> {
    Ok(History {
        alias: alias.to_string(),
        date: row.get("date")?,
        temperature_high: row.get("temp_high")?,
        temperature_low: row.get("temp_low")?,
        temperature_mean: row.get("temp_mean")?,
        dew_point: row.get("dew_point")?,
        humidity: row.get("humidity")?,
        precipitation_chance: row.get("precip_prob")?,
        precipitation_type: row.get("precip_type")?,
        precipitation_amount: row.get("precip")?,
        wind_speed: row.get("wind_speed")?,
        wind_gust: row.get("wind_gust")?,
        wind_direction: row.get("wind_dir")?,
        cloud_cover: row.get("cloud_cover")?,
        pressure: row.get("pressure")?,
        uv_index: row.get("uv_index")?,
        sunrise: row.get("sunrise_t")?,
        sunset: row.get("sunset_t")?,
        moon_phase: row.get("moon_phase")?,
        visibility: row.get("visibility")?,
        description: row.get("description")?,
    })
}

/// Get a summary of the weather history available for locations.
///
/// # Arguments
///
/// * `criteria` identifies the locations that should be used.
pub fn summary(
    conn: &mut Connection,
    weather_dir: &WeatherDir,
    filters: Option<Vec<LocationFilter>>,
) -> crate::Result<Vec<HistorySummaries>> {
    let db_sizes = query::db_size(&conn, "history")?;
    let history_counts = query::history_counts(&conn)?;
    let history_summaries = locations::get(&conn, filters)?
        .into_iter()
        .map(|location| {
            let db_size = db_sizes.get(&location.alias);
            let count = history_counts.get(&location.alias);
            let archive_size = weather_dir.archive(&location.alias).size() as usize;
            HistorySummaries {
                location,
                count,
                overall_size: Some(db_size + archive_size),
                raw_size: Some(db_size),
                store_size: Some(archive_size),
            }
        })
        .collect();
    Ok(history_summaries)
}

/// Reload a locations weather history for the *normalized* implementation of weather data.
///
/// # Argument
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather data directory.
/// * `alias` is the location that will be reloaded.
pub fn reload(conn: &mut Connection, weather_dir: &WeatherDir, alias: &str) -> crate::Result<()> {
    crate::log_elapsed_time!("reload");
    let size = estimate_size(&conn, "history")?;
    let lid = locations::location_id(conn, alias)?;
    let mut tx = create_tx!(conn, "failed to create reload transaction")?;
    delete(&tx, lid)?;
    metadata::delete(&tx, lid)?;
    for (md, history) in fs_lib::history_contents(weather_dir, alias)? {
        insert_history(&mut tx, lid, size, md.compressed_size as usize, &history)?;
    }
    commit_tx!(tx, "failed to commit reload for '{alias}'")
}

/// Delete the history for a specific location.
///
/// # Arguments
///
/// * `tx` is the transaction that will be used.
/// * `lid` is the database location id.
///
pub fn delete(tx: &Transaction, lid: i64) -> crate::Result<()> {
    crate::log_elapsed_time!("history delete");
    const DELETE_HISTORY: &str = r#"
        DELETE FROM history
        WHERE ROWID IN (
          SELECT h.ROWID FROM history AS h
          INNER JOIN metadata AS m ON h.mid = m.id
          WHERE m.lid = :lid
        )
    "#;
    let mut stmt = prepare_sql!(tx, DELETE_HISTORY, "failed to prepare delete SQL")?;
    execute_sql!(stmt, named_params! {":lid": lid}, "failed to delete history")?;
    Ok(())
}

pub fn check(conn: &mut Connection, weather_dir: &WeatherDir) -> crate::Result<Option<DbHistoryProblems>> {
    let fs_history_counts = fs_lib::get_history_counts(weather_dir, None)?;
    let mut history_problems = vec![];
    let mut detached_store = vec![];
    for history_summaries in summary(conn, weather_dir, None)? {
        match fs_history_counts.iter().find(|(l, _)| &history_summaries.location.alias == &l.alias) {
            None => detached_store.push(history_summaries),
            Some((_, fs_count)) => {
                if history_summaries.count != *fs_count {
                    history_problems.push(DbHistoryProblemDetails {
                        location: history_summaries.location,
                        db_histories: history_summaries.count,
                        fs_histories: *fs_count,
                    });
                }
            }
        }
    }
    match history_problems.len() > 0 || detached_store.len() > 0 {
        false => Ok(None),
        true => {
            let mut problems = DbHistoryProblems::default();
            if history_problems.len() > 0 {
                problems.history_problems.replace(history_problems);
            }
            if detached_store.len() > 0 {
                problems.detached_store.replace(detached_store);
            }
            Ok(Some(problems))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{
        // db::{admin::{create_db_admin, DbAdmin}, sqlite::{db_conn, prepare_sql, execute_sql}},
        db::{
            admin::{create_db_admin, DbAdmin},
            sqlite::db_conn,
        },
        testlib,
    };
    use chrono::NaiveDate;
    use std::{path::PathBuf, rc::Rc};

    #[test]
    fn add_history() {
        // use the database test resources
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("db"));
        let fixture_path = PathBuf::from(&fixture);

        // initialize the database
        let db_admin = Box::new(create_db_admin(Rc::new(WeatherDir::new(fixture_path.clone()).unwrap())));
        db_admin.history_init(false).unwrap();
        db_admin.history_load(3).unwrap();

        // set up the test environment
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
        let mut conn = db_conn!(weather_dir).unwrap();

        // verify there are no issues between the database and filesystem
        macro_rules! check {
            ($what:expr) => {
                if let Some(db_problems) = check(&mut conn, &weather_dir).unwrap() {
                    println!("{} problems: {:?}", $what, db_problems);
                    assert!(false);
                }
            };
        }
        check!("Init");

        // get the locations
        let mut locations = locations::get(&conn, None).unwrap();
        let south = locations.pop().unwrap();
        assert_eq!(south.alias, "south");
        let north = locations.pop().unwrap();
        assert_eq!(north.alias, "north");
        let between = locations.pop().unwrap();
        assert_eq!(between.alias, "between");

        // capture the history counts before adding any histories
        let before_history_counts = query::history_counts(&conn).unwrap();

        macro_rules! date {
            ($y:expr, $m:expr, $d:expr) => {
                NaiveDate::from_ymd_opt($y, $m, $d).unwrap()
            };
        }

        macro_rules! histories {
            ($location:expr, $start:expr, $end:expr) => {
                DateRange::new($start, $end)
                    .into_iter()
                    .map(|date| History { alias: $location.alias.clone(), date, ..Default::default() })
                    .collect::<Vec<_>>()
            };
        }

        // add histories that do not overlap
        let histories = histories!(north, date!(2025, 10, 1), date!(2025, 10, 31));
        let daily_histories = DailyHistories { location: north, histories };
        let add_count = add(&mut conn, &weather_dir, daily_histories).unwrap();
        assert_eq!(add_count, 31);
        check!("Add all");

        // capture the history counts after adding the histories
        let north_history_counts = query::history_counts(&conn).unwrap();
        assert_eq!(north_history_counts.get("north"), before_history_counts.get("north") + 31);

        // add histories that partially overlap
        let histories = histories!(south, date!(2015, 4, 7), date!(2015, 4, 21));
        let daily_histories = DailyHistories { location: south, histories };
        let add_count = add(&mut conn, &weather_dir, daily_histories).unwrap();
        assert_eq!(add_count, 7);
        check!("Partial add");

        // verify the history counts after partially adding the histories
        let south_history_counts = query::history_counts(&conn).unwrap();
        assert_eq!(south_history_counts.get("south"), before_history_counts.get("south") + 7);

        // add histories that all overlap
        let histories = histories!(between, date!(2015, 4, 1), date!(2015, 4, 14));
        let daily_histories = DailyHistories { location: between, histories };
        let add_count = add(&mut conn, &weather_dir, daily_histories).unwrap();
        assert_eq!(add_count, 0);
        check!("No add");

        // verify the history counts after trying to add
        let between_history_counts = query::history_counts(&conn).unwrap();
        assert_eq!(between_history_counts.get("between"), before_history_counts.get("between"));
    }
}
