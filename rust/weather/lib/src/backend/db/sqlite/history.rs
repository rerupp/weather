//! This module manages weather data history in the database.

mod query;
pub use query::history_dates;

use super::{
    commit_tx, create_tx, estimate_size, execute_sql, locations, metadata, prepare_cached_sql, prepare_sql, query_rows,
    SqlResult,
};
use crate::{
    backend::filesys::{HistoryArchive, WeatherDir},
    entities::{DailyHistories, DateRange, History, HistorySummaries, Location, LocationFilters},
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
    // make sure the database knows about the location
    let lid = locations::location_id(conn, &daily_histories.location.alias)?;

    // unfortunately the history archive does this when it adds
    daily_histories.histories.sort_by(|lhs, rhs| lhs.date.cmp(&rhs.date));
    daily_histories.histories.dedup_by(|lhs, rhs| lhs.date == rhs.date);

    // the history archive will make sure there are no duplicates added and issue log warnings
    let archive_file = weather_dir.archive(&daily_histories.location.alias);
    let archive = HistoryArchive::open(&daily_histories.location.alias, archive_file)?;
    let added_dates = archive.append(&daily_histories.histories)?;
    let added_histories = added_dates.len();

    // JIC
    if added_dates.len() == 0 {
        return Ok(0);
    }

    // remove the histories that were not added
    daily_histories.histories.retain(|history| added_dates.contains(&history.date));

    // for the database update, combine the histories and metadata
    let added_metadata = archive.metadata_by_dates(added_dates)?.collect::<Vec<_>>();
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
pub(super) fn insert_history(
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
    execute_sql!(stmt, params, "failed to insert history")
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
    filters: LocationFilters,
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
pub(super) fn reload(conn: &mut Connection, weather_dir: &WeatherDir, alias: &str) -> crate::Result<()> {
    crate::log_elapsed_time!("reload");
    let size = estimate_size(&conn, "history")?;
    let lid = locations::location_id(conn, alias)?;
    const SQL: &str = r#"
        DELETE FROM history
        WHERE ROWID IN (
          SELECT h.ROWID FROM history AS h
          INNER JOIN metadata AS m ON h.mid = m.id
          WHERE m.lid = :lid
        )
        "#;
    let mut tx = create_tx!(conn, "failed to create reload transaction")?;
    let mut stmt = prepare_sql!(tx, SQL, "failed to prepare delete SQL")?;
    execute_sql!(stmt, named_params! {":lid": lid}, "failed to delete history for '{alias}'")?;
    drop(stmt);
    metadata::delete(&tx, lid)?;
    for (md, history) in HistoryArchive::open(alias, weather_dir.archive(alias))?.metadata_and_history()? {
        insert_history(&mut tx, lid, size, md.compressed_size as usize, &history)?;
    }
    commit_tx!(tx, "failed to commit reload for '{alias}'")
}
