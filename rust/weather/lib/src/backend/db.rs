//! The database implementation of weather data.

pub(crate) mod admin;

// todo: filesys needs this right now, fix it
pub(in crate::backend) mod sqlite;

use super::LocationFilters;
use crate::backend::{filesys::WeatherDir, Backend, Config};

/// Create a database [`Backend`].
///
/// # Arguments
///
/// `config` is the weather data configuration.
pub(in crate::backend) fn create_db_backend(config: Config) -> crate::Result<Box<dyn Backend>> {
    log::debug!("Database data adapter");
    let weather_dir = WeatherDir::try_from(&config)?;
    Ok(Box::new(sqlite::SqliteBackend::new(config, weather_dir)))
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
