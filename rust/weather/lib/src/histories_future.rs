/// The new histories module manages getting weather history from Weather providers.
///
/// The weather data providers currently provide access via the network and `REST`
/// services. In order to help applications be responsive the services are run in
/// a worker thread. The [HistoriesFuture] struct hides how that happens and allows
/// an application to poll for data and not pause while the data is being retrieved.
mod timeline_client;
use timeline_client::TimelineClient;

use crate::prelude::{Configuration, DailyHistories, DateRange, History, HistoryDates, Location};
use std::fmt::Formatter;
use std::time::Duration;
use std::{
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::Instant,
};

/// Consolidate error creation to this macro.
macro_rules! err {
    ($reason:expr) => {
        Err(crate::Error::from($reason))
    };
}

/// Start the thread that will collect historical weather data.
///
/// # Arguments
///
/// * `dates` identify the historical weather data to collect.
/// * `history_dates` is the current location history dates.
/// * `config` contains the current weather data configuration information.
///
pub fn get(dates: DateRange, history_dates: HistoryDates, config: &Configuration) -> crate::Result<HistoriesFuture> {
    for date in &dates {
        if history_dates.history_dates.iter().any(|date_range| date_range.contains(&date)) {
            Err("The location already has histories for those dates.")?;
        }
    }
    HistoriesFuture::new(history_dates.location, dates, config, None)
}

/// The historical weather data collector and thread manager.
pub struct HistoriesFuture {
    /// The location getting new historical weather data.
    location: Location,
    /// The result of collecting historical weather data.
    outcome: Arc<HistoryOutcome>,
    /// The thread being used to access the remote server.
    client_handle: JoinHandle<()>,
    /// When the background thread was started.
    start: Instant,
    /// How long to wait for the remote server to respond before considering it has timed out.
    timeout: f64,
}
impl std::fmt::Debug for HistoriesFuture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HistoriesFuture({})", self.location)
    }
}
impl HistoriesFuture {
    /// Create the background thread and begin collection of historical weather data. The
    /// background thread uses a [`TimelineClient`] to collect data from the *Visual
    /// Crossing* server.
    ///
    /// #Arguments
    ///
    /// * `location` is the location getting new historical weather data.
    /// * `dates` identify the historical weather data to collect.
    /// * `config` contains the current weather data configuration information.
    /// * `timeout` is the number of seconds to wait for a remote server response before considering it has timed out.
    ///
    fn new(
        location: Location,
        dates: DateRange,
        config: &Configuration,
        timeout: Option<usize>,
    ) -> crate::Result<Self> {
        let outcome = Arc::new(HistoryOutcome::new());
        let timeline_client =
            TimelineClient::new(location.clone(), dates, &config.visual_crossing.endpoint, outcome.clone())?;
        let api_key = config.visual_crossing.api_key.clone();
        let client_handle = thread::spawn(move || {
            timeline_client.execute(api_key);
        });
        let timeout = timeout.map_or(30.0, |to| to as f64);
        Ok(Self { outcome, location, client_handle, start: Instant::now(), timeout })
    }
    /// Query the background thread to see if it has complected.
    ///
    pub fn is_finished(&self) -> bool {
        if (Instant::now() - self.start).as_secs_f64() <= self.timeout {
            self.client_handle.is_finished()
        } else {
            self.outcome.set(HistoriesResult::Timeout);
            true
        }
    }
    /// Retrieve the result of collecting the historical weather data. This call guarantees
    /// the background thread has completed or has timed out communicating with the
    /// remote server.
    ///
    pub fn get(&self) -> crate::Result<Option<DailyHistories>> {
        // guard against the thread being a runaway for whatever reason
        if !self.outcome.is_timeout() {
            loop {
                if self.is_finished() {
                    break;
                }
                thread::sleep(Duration::from_millis(1));
            }
        }
        match self.outcome.get() {
            None => Ok(None),
            Some(histories) => match histories {
                HistoriesResult::Error(error) => err!(error),
                HistoriesResult::Timeout => err!("The timeline client timed out."),
                HistoriesResult::Histories(histories) => {
                    Ok(Some(DailyHistories { location: self.location.clone(), histories }))
                }
            },
        }
    }
}

/// The result of accessing the remote historical weather data server.
#[derive(Debug)]
pub enum HistoriesResult {
    /// An error occurred accessing the server.
    Error(String),
    /// A response was not received from the remote server.
    Timeout,
    /// The collection of historical weather data returned from the server.
    Histories(Vec<History>),
}
impl HistoriesResult {
    /// Test if the remote server timed out.
    pub fn is_timeout(&self) -> bool {
        matches!(self, HistoriesResult::Timeout)
    }
}

/// Manages multi-thread access to the result of collecting data from the remote server.
struct HistoryOutcome {
    /// The outcome of communicating with the remote server.
    outcome: RwLock<Option<HistoriesResult>>,
}
impl HistoryOutcome {
    /// The outcome is initially `None` indicating the server has not yet responded.
    fn new() -> HistoryOutcome {
        Self { outcome: RwLock::new(Default::default()) }
    }
    /// Test if the remote server has timed out.
    fn is_timeout(&self) -> bool {
        self.outcome.read().unwrap().as_ref().map_or(false, |outcome| outcome.is_timeout())
    }
    /// Get the outcome of collecting data from the remote server. If `None` is returned the
    /// remote server has not responded.
    fn get(&self) -> Option<HistoriesResult> {
        self.outcome.write().unwrap().take()
    }
    /// Set the outcome of collecting data from the remote server.
    ///
    /// #Arguments
    ///
    /// `result` identifies that state of data collection from the remote server.
    ///
    fn set(&self, result: HistoriesResult) {
        self.outcome.write().unwrap().replace(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn history_outcome() {
        let testcase = HistoryOutcome::new();
        assert!(testcase.get().is_none());
        testcase.set(HistoriesResult::Timeout);
        assert!(testcase.is_timeout());
        assert!(matches!(testcase.get(), Some(HistoriesResult::Timeout)));
        assert!(!testcase.is_timeout());
        assert!(testcase.get().is_none());
    }
}
