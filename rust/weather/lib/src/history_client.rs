//! The source of weather history for locations.

use crate::{
    backend::Config,
    entities::{DailyHistories, DateRange, Location},
    Result,
};
use std::fmt::Debug;
use timeline_client::TimelineClient;

mod rest_client;

mod timeline_client;

/// Creates a history client.
///
/// # Arguments
///
/// - `config` is the weather data configuration.
///
pub fn create_history_client(config: &Config) -> Result<Box<dyn HistoryClient>> {
    // currently there is only 1 client so just create it.
    match TimelineClient::new(config) {
        Ok(history_client) => Ok(Box::new(history_client)),
        Err(error) => Err(error),
    }
}

/// The internal API used to get location weather history.
///
pub trait HistoryClient: Debug + Send {
    /// Execute the request to get history for a location.
    ///
    /// # Arguments
    ///
    /// * `location` identifies what weather history to get.
    /// * `date_range` controls the weather history dates.
    ///
    fn execute(&self, location: &Location, date_range: &DateRange) -> Result<()>;
    /// Query if the request has finished or return an error if there is no active request. `Ok(true)`
    /// guarantees the request response is available.
    ///
    fn poll(&self) -> Result<bool>;
    /// Get the request result by blocking until it finishes.
    ///
    fn get(&self) -> Result<DailyHistories>;
}
