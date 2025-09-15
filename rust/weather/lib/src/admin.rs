//! The weather data administration API and data beans.

pub use crate::backend::admin::{create_weather_admin, WeatherAdmin};

/// The administration `stat` information.
#[derive(Debug)]
pub struct Components {
    /// The database information.
    pub db_details: Option<DbDetails>,
    /// The archive information.
    pub fs_details: FilesysDetails,
}

/// The database information.
#[derive(Debug)]
pub struct DbDetails {
    /// The size of the database.
    pub size: usize,
    /// The location weather history information.
    pub location_details: Vec<LocationDetails>,
}

/// Information about the weather history archives.
#[derive(Debug, Default)]
pub struct FilesysDetails {
    /// The total size of weather history archives.
    pub size: usize,
    /// The location information
    pub location_details: Vec<LocationDetails>,
}

/// Weather history metadata for a [location](crate::prelude::Location).
#[derive(Debug)]
pub struct LocationDetails {
    /// The location alias name.
    pub alias: String,
    /// The number of bytes being used to hold weather history information.
    pub size: usize,
    /// The count of weather histories the [location](crate::prelude::Location) has available.
    pub histories: usize,
}

#[derive(Debug)]
pub struct UsCityDetails {
    pub db_size: usize,
    pub state_info: Vec<(String, usize)>,
}
