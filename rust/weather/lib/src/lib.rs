//! A library containing the weather data backend API and implementation.
//!
//! This implementation is loosely base on a `Python` project I created several years ago. When
//! I started the `Python` project I wanted to chart historical weather information for different
//! areas we were interested in spending the winter. The idea of building a CLI based on the
//! original weather data implementation seemed like a fun journey.
//!
//! # History
//!
//! The `Python` weather data project is based on *DarkSky* weather history data.
//! Unfortunately the API was purchased by Apple and is no longer publicly available (or at least
//! free) but I had collected years of data for a dozen or more sites. Initially the `Rust` implementation
//! faithfully followed the `Python` implementation using the *DarkSky* data.
//!
//! # October 2023 Version
//!
//! Late summer I came across *Visual Crossings* and their *Timeline* weather history API. It had
//! most of the historical weather data I was interested in, so I decided to support adding weather
//! history using their API. The biggest change behind this move was storing weather history in a new
//! `JSON` document format. Both *DarkSky* and *Timeline* historical data are supersets of the data
//! currently being stored. I decided this was the best approach in case *Timeline* goes away and a new
//! weather history API needs to be used.
//!
//! # September 2024 Version
//!
//! Over the course of this release it became apparent that the `db` and `filesys` source files were getting
//! out of hand in size. With that in mind, modules have been moved into individual source files. Source
//! files still continue to have modules but not like the previous version.
//!
//! The Visual Crossing Rest client was moved out of the [backend] module. It has been separated into a general
//! Rest client and a timeline API specific client. The plan is to hook up my weather station at the house
//! to track specifically what my weather is. I really might have too much time on my hands...
//!
//! When the `admin` API was brought back into the main `CLI` the associated modules from that binary were
//! mostly dropped into the library. This is one of the areas that will get attention if `admin` commands
//! are move into the TUI.
//!

// Ignore broke links due to --document-private-items not being used.
#![allow(rustdoc::private_intra_doc_links)]

/// The library result.
pub type Result<T> = std::result::Result<T, Error>;

/// The library error.
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

mod history_client;

mod admin;

/// The public weather data API.
pub mod prelude {
    pub use crate::{
        entities::{
            DailyHistories, DateRange, DateRanges, History, HistoryDates, HistorySummaries, HistorySummary, Location,
            LocationFilter, LocationFilters, State, CityFilter,
        },
        history_client::HistoryClient,
        location_filter, location_filters,
        weather_data::{create_weather_data, WeatherData},
    };
}

/// The public administration weather data API.
pub mod admin_prelude {
    pub use crate::admin::{
        create_weather_admin, Components, DbDetails, FilesysDetails, LocationDetails, UsCityDetails, WeatherAdmin,
    };
}

struct LogElapsedTime {
    description: String,
    start: std::time::Instant,
    log_level: log::Level,
}
impl LogElapsedTime {
    pub fn new(description: &str, log_level: Option<log::Level>) -> Self {
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
