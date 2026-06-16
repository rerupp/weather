//! The library that defines the weather history API and data implementations.
//!
//! # Overview
//!
//! The library defines and implements two APIs.
//!
//! * [weather data](../weather_data/WeatherData) provides the API to manage weather history data
//! and extract details about the weather history that has been collected..
//! * [weather administration](../admin/WeatherAdmin) provides an administrative level API to perform
//! tasks like initialize weather history storage.
//!
//! Weather history is associated with locations. Weather history for a location is retrieved
//! using latitude and longitude coordinates. It can be thought of as a city or municipality however
//! it could also be a home address. Really anything with a latitude and longitude.
//!
//! All locations have a unique user defined `alias` name. They also are associated with a
//! timezone allowing properties such as sunrise and sunset to be shown in the locations
//! localtime..
//!
//! All modules in the library use the library [Result] for error conditions. The [prelude] module
//! exports the weather data structures and functions. The [admin_prelude] module exports the
//! administrative data structures and functions.
//!

// Ignore broke links due to --document-private-items not being used.
#![allow(rustdoc::private_intra_doc_links)]

/// The library result.
///
pub type Result<T> = std::result::Result<T, Error>;

/// The library error.
///
#[derive(Debug)]
pub struct Error(String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<String> for Error {
    /// Create an error from the provided string.
    fn from(error: String) -> Self {
        Error(error)
    }
}
impl From<&str> for Error {
    /// Create an error from the provided string.
    fn from(error: &str) -> Self {
        Error(error.to_string())
    }
}

mod weather_data;
pub use weather_data::create_weather_data;

mod backend;
mod entities;
mod admin;
mod configuration;
mod histories_future;

pub mod prelude {
    //! The weather data user level API and data structure exports.
    //!
    pub use crate::{
        configuration::Configuration,
        entities::{
            City, DailyHistories, DatabaseHistorySummary, DateRange, DateRanges, FilesysHistorySummary, History,
            HistoryDates, HistorySummary, Location, LocationFilter, State,
        },
        histories_future::HistoriesFuture,
        weather_data::{create_weather_data, WeatherData},
    };
}

pub mod admin_prelude {
    //! The weather data administration level API and data structure exports.
    //!
    pub use crate::admin::{
        entities::{
            CitiesDetails, Components, CountryDetails, DbDetails, DbHistoryProblems, DbLocationProblems, DbProblems,
            FilesysDetails, FilesysDocumentProblem, FilesysLocationProblem, FilesysProblems, LocationDetails,
            RegionDetails,
        },
        WeatherAdmin,
    };
}

#[doc(hidden)]
struct LogElapsedTime {
    description: String,
    start: std::time::Instant,
    log_level: log::Level,
}
impl LogElapsedTime {
    pub fn new(description: impl ToString, log_level: Option<log::Level>) -> Self {
        Self {
            description: description.to_string(),
            start: std::time::Instant::now(),
            log_level: log_level.unwrap_or(log::Level::Debug),
        }
    }
}
impl Drop for LogElapsedTime {
    fn drop(&mut self) {
        let micros = (std::time::Instant::now() - self.start).as_micros();
        match micros < 1_000 {
            true => log::log!(self.log_level, "{} {}us", self.description, micros),
            false => log::log!(self.log_level, "{} {}ms", self.description, toolslib::fmt::commafy(micros / 1_000)),
        };
    }
}

#[doc(hidden)]
macro_rules! log_elapsed_time {
    (info, $description:expr) => {
        let __log_elapsed_time_instance__ = $crate::LogElapsedTime::new($description, Some(log::Level::Info));
    };
    ($description:expr) => {
        let __log_elapsed_time_instance__ = $crate::LogElapsedTime::new($description, None);
    };
    (trace, $description:expr) => {
        let __log_elapsed_time_instance__ = $crate::LogElapsedTime::new($description, Some(log::Level::Trace));
    };
}
use log_elapsed_time;
