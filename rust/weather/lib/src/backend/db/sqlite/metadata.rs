//! This module manages the metadata surrounding weather data history.
//!
use super::{execute_sql, prepare_cached_sql, prepare_sql};
use chrono::NaiveDate;
use rusqlite::{named_params, Transaction};

pub const TABLE_NAME: &'static str = "metadata";

/// Create a database metadata specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("metadata {}", format!($($arg)*)))
    }
}

/// Create an error from the metadata specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// Insert metadata into the database.
///
/// # Arguments
///
/// * `tx` is the transaction that will be used to insert data.
/// * `lid` is the location primary id.
/// * `date` is the history date.
/// * `store_size` is the size of data in the database.
/// * `size` is the size of history data.
///
pub fn insert(tx: &Transaction, lid: i64, date: &NaiveDate, store_size: usize, size: usize) -> crate::Result<i64> {
    const METADATA_SQL: &'static str = r#"
        INSERT INTO metadata (lid, date, store_size, size)
            VALUES (:lid, :date, :store_size, :size)
    "#;
    let mut stmt = prepare_cached_sql!(tx, METADATA_SQL, "failed to prepare insert SQL")?;

    let params = named_params![":lid": lid,":date": date,":store_size": store_size,":size": size];
    execute_sql!(stmt, params, "failed to insert data for lid={lid} on {date}")?;
    Ok(tx.last_insert_rowid())
}

/// Remove all metadata associated with a location id.
///
/// # Arguments
///
/// * `tx` is the database transaction that will be used.
/// * `lid` is the location id.
///
pub fn delete(tx: &Transaction, lid: i64) -> crate::Result<()> {
    const SQL: &str = "DELETE FROM metadata where lid=:lid";
    let mut stmt = prepare_sql!(tx, SQL, "failed to prepare delete SQL")?;
    execute_sql!(stmt, named_params! {":lid": lid}, "failed to delete metadata for lid={lid}")
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         backend::{
//             db::sqlite::db_conn,
//             filesys::WeatherDir
//         },
//         entities::{DailyHistories, History, Location},
//     };
//     use toolslib::date_time::get_date;
//
//     #[test]
//     fn full_monty() {
//         let weather_dir = WeatherDir::try_from("../weather_data").unwrap();
//         let conn = db_conn!(&weather_dir).unwrap();
//         let mut locations = locations::get(&conn, &vec!["kfalls".to_string()]).unwrap();
//         let location = locations.pop().unwrap();
//         let alias = location.alias.clone();
//         let daily_histories = DailyHistories {
//             location,
//             histories: vec![
//                 History {alias: alias.clone(), date: get_date(2024, 2, 27), ..Default::default()},
//                 History {alias: alias.clone(), date: get_date(2024, 2, 28), ..Default::default()},
//                 History {alias: alias.clone(), date: get_date(2024, 3, 1), ..Default::default()},
//                 History {alias: alias.clone(), date: get_date(2024, 3, 2), ..Default::default()},
//             ],
//         };
//         let (lid, histories) = examine_add_histories(&conn, &daily_histories).unwrap();
//         assert_eq!(histories.len(), 2);
//         assert_eq!(lid, 6);
//         assert_eq!(histories[0].date, get_date(2024, 2, 27));
//         assert_eq!(histories[1].date, get_date(2024, 2, 28));
//     }
// }
