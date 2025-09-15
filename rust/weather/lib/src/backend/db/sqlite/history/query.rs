//! The common weather database queries.
//!

use super::{locations, metadata, prepare_sql, query_rows, SqlResult};
use crate::entities::{DateRanges, HistoryDates, LocationFilters};
use chrono::NaiveDate;
use rusqlite::{named_params, Connection, Row};
use sql_query_builder as sql;
use std::collections::HashMap;

/// Create a database locations specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("Query {}", format!($($arg)*)))
    }
}

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// Get the location history dates.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `criteria` is the location data criteria.
///
pub fn history_dates(conn: &Connection, filters: LocationFilters) -> crate::Result<Vec<HistoryDates>> {
    // collect up the locations that match the criteria
    let locations = locations::get(conn, filters)?;

    let mut history_dates = vec![];
    // if the data criteria didn't match anything don't bother with a query
    if locations.len() > 0 {
        let aliases = locations.iter().map(|location| location.alias.as_str()).collect();
        let mut alias_date_ranges = query_history_dates(conn, aliases)?;

        for location in locations {
            if let Some(date_ranges) = alias_date_ranges.remove(&location.alias) {
                history_dates.push(HistoryDates { location, history_dates: date_ranges.date_ranges })
            } else {
                history_dates.push(HistoryDates { location, history_dates: vec![] })
            }
        }
    }
    Ok(history_dates)
}

/// Execute the query to get location history dates.
///
/// # Arguments
///
/// * `conn` is the database connection used to execute the query.
/// * `aliases` is used to restrict what locations will be returned.
///
fn query_history_dates(conn: &Connection, aliases: Vec<&str>) -> crate::Result<HashMap<String, DateRanges>> {
    // build the history date query
    let mut query = sql::Select::new()
        .select("l.alias AS alias, m.date AS date")
        .from("locations AS l")
        .inner_join("metadata AS m ON l.id = m.lid")
        .order_by("l.alias, m.date");
    for alias in aliases {
        query = query.where_or(&format!("l.alias = '{}'", alias));
    }

    // execute the query
    let mut stmt = prepare_sql!(conn, &query.to_string(), "failed to prepare history dates query")?;
    let mut rows = query_rows!(stmt, [], "failed to query history dates")?;

    // collect the location dates
    let mut location_dates: Vec<(String, Vec<NaiveDate>)> = vec![];
    loop {
        match rows.next() {
            Err(error) => err!("failed to get history date row: {:?}", error)?,
            Ok(None) => break,
            Ok(Some(row)) => {
                // capture any errors getting row content
                #[inline]
                fn next_history_date(row_: &Row) -> SqlResult<(String, NaiveDate)> {
                    Ok((row_.get(0)?, row_.get(1)?))
                }
                match next_history_date(row) {
                    Err(error) => err!("failed to get history date from row: {:?}", error)?,
                    Ok((alias, date)) => match location_dates.last_mut() {
                        None => location_dates.push((alias, vec![date])),
                        Some((current_alias, dates)) => {
                            if current_alias == &alias {
                                dates.push(date);
                            } else {
                                location_dates.push((alias, vec![date]));
                            }
                        }
                    },
                }
            }
        }
    }

    // return the results as a map of alias name and date ranges
    let alias_date_ranges = location_dates
        .into_iter()
        .map(|(alias, dates)| {
            let date_ranges = DateRanges::new(&alias, dates);
            (alias, date_ranges)
        })
        .collect();
    Ok(alias_date_ranges)
}

/// Calculate the amount of space being used by a table.
///
/// This is terribly expensive but it suffices for right now
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `table_name` is the table whose space will be calculated.
///
pub fn db_size(conn: &Connection, table_name: &str) -> crate::Result<DbSizes> {
    // get the count of history dates for each location
    let history_counts = history_counts(conn)?;
    let total = history_counts.0.iter().map(|(_, size)| size).sum::<usize>();

    // get the overall size of history in the database
    let table_size_result = if table_name == metadata::TABLE_NAME {
        sqlite_metadata_size(conn)
    } else {
        sqlite_history_size(conn, table_name)
    };
    let table_size = match table_size_result {
        Ok(table_size) => table_size,
        Err(error) => err!("failed to get table size: {:?}", error)?,
    };

    // calculate the sizes based on the number of histories
    let locations_size: Vec<(String, usize)> = history_counts
        .0
        .into_iter()
        .map(|(alias, count)| {
            let percentage = count as f64 / total as f64;
            let size = (table_size as f64 * percentage) as usize;
            (alias, size)
        })
        .collect();
    Ok(DbSizes(locations_size))
}

/// The collection of location aliases and the size in the database.
#[derive(Debug)]
pub struct DbSizes(
    /// The location and database size tuples.
    Vec<(String, usize)>,
);
impl DbSizes {
    /// Get the size of history in the database for a location.
    ///
    /// # Arguments
    ///
    /// * `alias` is the location alias name.
    ///
    pub fn get(&self, alias: &str) -> usize {
        for (location_alias, size) in &self.0 {
            if alias == location_alias {
                return *size;
            }
        }
        // this will happen if the location does not have history
        0
    }
}

/// Used internally to help calculate the amount of metadata space being used by locations.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
fn sqlite_metadata_size(conn: &Connection) -> crate::Result<usize> {
    const SQL: &str = r#"
        SELECT
            SUM(pgsize) AS size
        FROM dbstat
            WHERE name LIKE '%metadata%'
    "#;
    let mut stmt = prepare_sql!(conn, SQL, "failed to prepare metadata size query")?;
    let db_size = stmt.query_row([], |row| {
        let size: usize = row.get("size")?;
        Ok(size)
    });
    match db_size {
        Ok(db_size) => Ok(db_size),
        Err(error) => err!("failed to get metadata database size: {:?}", error)?,
    }
}

/// Used internally to help calculate the amount of history space being used by locations.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `table_name` is the table name that will be examined.
///
fn sqlite_history_size(conn: &Connection, table_name: &str) -> crate::Result<usize> {
    // todo: is it right to include both history and metadata here?
    const SQL: &str = r#"
        SELECT
            SUM(pgsize) AS size
        FROM dbstat
            WHERE name LIKE :table OR name LIKE '%metadata%'
        "#;
    let mut stmt = prepare_sql!(conn, SQL, "failed to prepare history size query")?;
    let db_size = stmt.query_row(named_params! {":table": format!("%{}%", table_name)}, |row| {
        let size: usize = row.get("size")?;
        Ok(size)
    });
    match db_size {
        Ok(db_size) => Ok(db_size),
        Err(error) => err!("failed to get history database size: {:?}", error)?,
    }
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
pub fn history_counts(conn: &Connection) -> crate::Result<HistoryCounts> {
    // query the history counts
    const SQL: &str = r#"
        SELECT
            l.alias AS alias,
            COUNT(m.date) AS COUNT
        FROM locations AS l
            INNER JOIN metadata AS m ON l.id=m.lid
        GROUP BY l.alias
        ORDER BY l.alias
        "#;
    let mut stmt = prepare_sql!(conn, SQL, "failed to prepare history counts query")?;
    let mut rows = query_rows!(stmt, [], "failed to get history counts")?;

    let mut counts: Vec<(String, usize)> = vec![];
    loop {
        match rows.next() {
            Err(error) => err!("failed to get next history count: {:?}", error)?,
            Ok(None) => break,
            Ok(Some(row)) => {
                #[inline]
                fn next_alias_count(row_: &Row) -> SqlResult<(String, usize)> {
                    Ok((row_.get("alias")?, row_.get("count")?))
                }
                match next_alias_count(row) {
                    Ok(alias_count) => counts.push(alias_count),
                    Err(_) => {}
                }
            }
        }
    }
    Ok(HistoryCounts(counts))
}

/// The collection of location aliases and history counts.
#[derive(Debug)]
pub struct HistoryCounts(
    /// The location and history count tuples.
    Vec<(String, usize)>,
);
impl HistoryCounts {
    /// Get the history count for a location.
    ///
    /// # Arguments
    ///
    /// * `alias` is the location alias name.
    ///
    pub fn get(&self, alias: &str) -> usize {
        self.0
            .iter()
            .find_map(|(inner_alias, count)| match inner_alias == alias {
                true => Some(*count),
                false => None,
            })
            // if there are no histories for the location, none have been added
            .unwrap_or(0)
    }
}
