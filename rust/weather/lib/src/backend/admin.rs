//! The administration commands are scoped to this module.
use super::{db, filesys};
use crate::{
    admin_prelude::{Components, UsCityDetails},
    entities::LocationFilters,
};
use std::path::PathBuf;
use toolslib::{fmt::commafy, stopwatch::StopWatch};

/// Create an instance of the weather data administration `API`.
///
/// # Arguments
///
/// * `dirname` is the weather data directory pathname.
pub fn create_weather_admin(dirname: Option<PathBuf>) -> crate::Result<WeatherAdmin> {
    let dirname = dirname.map_or(Default::default(), |pb| pb.as_path().display().to_string());
    WeatherAdmin::new(dirname.as_str())
}

/// The weather data administration `API`.
#[derive(Debug)]
pub struct WeatherAdmin(
    /// The weather data directory.
    filesys::WeatherDir,
);
impl WeatherAdmin {
    /// Create an instance of the weather data administration `API`.
    ///
    /// # Arguments
    ///
    /// * `dirname` is the weather data directory pathname.
    fn new(dirname: &str) -> crate::Result<Self> {
        Ok(WeatherAdmin(filesys::create_weather_dir(dirname)?))
    }

    /// Initialize the weather database using the supplied database configuration.
    ///
    /// # Arguments
    ///
    /// * `drop` when `true` will delete the schema before initialization.
    /// * `load` when `true` will load weather data into the database.
    pub fn init(&self, drop: bool, load: bool, threads: usize) -> crate::Result<()> {
        db::admin::init_db(&self.0, drop, load, threads)?;
        Ok(())
    }

    /// Deletes the weather database schema and optionally deletes the database.
    ///
    /// # Arguments
    ///
    /// * `delete` when `true` will delete the database file.
    pub fn drop(&self, delete: bool) -> crate::Result<()> {
        db::admin::drop_db(&self.0, delete)?;
        Ok(())
    }

    /// Provides information about the weather data archives and database.
    pub fn components(&self) -> crate::Result<Components> {
        let fs_details = filesys::admin::filesys_details(&self.0)?;
        let db_details = db::admin::db_details(&self.0)?;
        Ok(Components { db_details, fs_details })
    }

    /// Reload history for locations.
    ///
    /// # Arguments
    ///
    /// * `filters` identifies the locations that will be reloaded.
    ///
    pub fn reload(&self, filters: LocationFilters) -> crate::Result<usize> {
        let locations = db::admin::reload(&self.0, filters)?;
        Ok(locations.len())
    }

    /// Load the US Cities database.
    ///
    /// # Arguments
    ///
    /// * `uscities_path` is the US Cities `CSV` file that will populate the database.
    pub fn uscities_load(&self, uscities_path: &PathBuf) -> crate::Result<()> {
        let stopwatch = StopWatch::start_new();
        let count = db::admin::uscities_load(&self.0, uscities_path)?;
        log::debug!("Loaded {} US Cities in {}", commafy(count), stopwatch);
        Ok(())
    }

    /// Delete the US Cities database.
    pub fn uscities_delete(&self) -> crate::Result<()> {
        db::admin::uscities_delete(&self.0)?;
        Ok(())
    }

    /// Show information about the US Cities database.
    pub fn uscities_info(&self) -> crate::Result<UsCityDetails> {
        let cities_info = db::admin::uscities_info(&self.0)?;
        Ok(cities_info)
    }
}
