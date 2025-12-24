//! The database implementation of weather data.

pub(crate) mod admin;

// todo: filesys needs this right now, fix it
pub(in crate::backend) mod sqlite;

use crate::backend::{filesys::WeatherDir, Backend, Configuration};
use std::sync::Arc;

/// Create a database [`Backend`].
///
/// # Arguments
///
/// `config` is the weather data configuration.
pub(in crate::backend) fn create_db_backend(configuration: Arc<Configuration>) -> crate::Result<Box<dyn Backend>> {
    log::debug!("Database data adapter");
    Ok(Box::new(sqlite::SqliteBackend::new(configuration)?))
}

/// Tests if the database has been initialized.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
///
pub fn is_available(weather_dir: &WeatherDir) -> bool {
    sqlite::db_exists(weather_dir)
}
