//! The database implementation of weather data.
//! 
//! The [sqlite] module contains the database implementation of weather history. As the name
//! suggests it is built using Sqlite3.

pub(crate) mod admin;

mod sqlite;

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
    sqlite::weather::db_exists(weather_dir)
}

#[derive(Debug, Default)]
pub struct DbMetadata {
    #[allow(unused)]
    pub table: String,
    pub data_size: usize,
    pub data_unused: usize,
    pub index_size: usize,
    pub index_unused: usize,
}
// todo: remove allow(unused)
#[allow(unused)]
impl DbMetadata {
    fn size(&self) -> usize {
        self.data_size + self.index_size
    }
    fn unused(&self) -> usize {
        self.data_unused + self.index_unused
    }
}
