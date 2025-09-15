//! The `Python` weather library [HistoryClient] wrapper.

use super::*;
use py_entities::*;
use weather_lib::prelude::HistoryClient;

#[pyclass]
pub struct PyHistoryClient {
    history_client: Box<dyn HistoryClient>,
}
impl PyHistoryClient {
    pub fn new(history_client: Box<dyn HistoryClient>) -> Self {
        Self { history_client }
    }
}
/// The internal API used to get location weather history.
///
#[pymethods]
impl PyHistoryClient {
    /// Execute the request to get history for a location.
    ///
    /// # Arguments
    ///
    /// * `location` identifies what weather history to get.
    /// * `date_range` controls the weather history dates.
    ///
    fn execute(&self, location: PyLocation, date_range: PyDateRange) -> PyResult<()> {
        match self.history_client.execute(&location.into(), &date_range.into()) {
            Err(error) => system_err!(error.to_string()),
            Ok(_) => Ok(()),
        }
    }
    /// Query if the request has finished or return an error if there is no active request. `Ok(true)`
    /// guarantees the request response is available.
    ///
    fn poll(&self) -> PyResult<bool> {
        match self.history_client.poll() {
            Err(error) => system_err!(error.to_string()),
            Ok(completed) => Ok(completed),
        }
    }
    /// Get the request result by blocking until it finishes.
    ///
    fn get(&self) -> PyResult<PyDailyHistories> {
        match self.history_client.get() {
            Err(error) => system_err!(error.to_string()),
            Ok(daily_histories) => Ok(daily_histories.into()),
        }
    }
}
