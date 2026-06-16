//! Load weather history from the filesystem archives.
//!
//! The loader is designed to initialize a database. It uses the [filesystem get]( fs_lib::history_contents::get)
//! function to retrieve all locations weather history in the filesystem. The loader only inserts data,
//! it will not attempt to update or ignore existing weather history.
//!
use crate::backend::{
    db::sqlite::weather,
    filesys::{fs_lib, WeatherDir},
};
use rusqlite::Connection;
use std::collections::HashMap;

/// Create an error from the load history specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error::from(format!("history_loader {}", format!($($arg)*))))
    };
}

/// Retrieve weather history for all locations and add it to the database.
///
/// # Argument
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather data directory.
/// * `threads` is the number of workers to use getting data from archives.
///
pub fn load(mut conn: Connection, weather_dir: &WeatherDir, max_threads: usize) -> crate::Result<()> {
    let alias_ids = weather::locations::id_aliases(&conn)?
        .into_iter()
        .map(|(id, alias)| (alias, id))
        .collect::<HashMap<String, i64>>();

    // create the transaction
    let mut tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(error) => err!("consumer failed to create transaction: {:?}", error)?,
    };
    let timer = toolslib::stopwatch::StopWatch::start_new();
    let mut count: usize = 0;
    let insert_mgr = weather::history::InsertMgr::new();
    for (metadata, history) in fs_lib::history_contents::get(weather_dir, None, Some(max_threads))? {
        match alias_ids.get(&history.alias) {
            None => {
                log::error!("Did not find location alias {}", history.alias);
            }
            Some(lid) => {
                insert_mgr.insert(&mut tx, *lid, metadata, history)?;
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
