//! The normalized database archive loader for [History].
//!
use super::{history::insert_history, locations};
use crate::backend::{
    db::sqlite::estimate_size,
    filesys::{fs_lib, WeatherDir},
};
use rusqlite::Connection;
use std::collections::HashMap;

/// Create a load history specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("loader {}", format!($($arg)*)))
    }
}

/// Create an error from the load history specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// Take the [History] archives and push them into the database.
///
/// # Argument
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather data directory.
/// * `threads` is the number of workers to use getting data from archives.
///
pub fn load(mut conn: Connection, weather_dir: &WeatherDir, max_threads: usize) -> crate::Result<()> {
    let size_estimate = estimate_size(&conn, "history")?;
    let alias_ids =
        locations::id_aliases(&conn)?.into_iter().map(|(id, alias)| (alias, id)).collect::<HashMap<String, i64>>();

    // create the transaction
    let mut tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(error) => err!("consumer failed to create transaction: {:?}", error)?,
    };
    let timer = toolslib::stopwatch::StopWatch::start_new();
    let mut count: usize = 0;
    for (metadata, history) in fs_lib::get_history_contents(weather_dir, None, Some(max_threads))? {
        match alias_ids.get(&history.alias) {
            None => {
                log::error!("Did not find location alias {}", history.alias);
            }
            Some(lid) => {
                let mut size = size_estimate + history.description.as_ref().map_or(0, |s| s.len());
                size += history.precipitation_type.as_ref().map_or(0, |t| t.len());
                insert_history(&mut tx, *lid, size, metadata.compressed_size as usize, &history)?;
                count += 1;
            }
        }
    }
    let elapsed = timer.elapsed().as_millis();
    let per_msec = count as f64 / elapsed as f64;
    if let Err(error) = tx.commit() {
        err!("failed to commit transaction: {:?}", error)?;
    }
    log::debug!("{count} histories loaded ({per_msec:.1}/msec)");
    Ok(())
}
