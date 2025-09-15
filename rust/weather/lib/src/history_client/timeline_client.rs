//! The Visual Crossing weather data services client.
use super::{rest_client::{RestClient, RestClientHandle, RestClientResult}, HistoryClient};
use crate::{
    backend::Config,
    prelude::{DailyHistories, DateRange, History, Location},
    Error, Result
};
use chrono::DateTime;
use reqwest::{
    // use the blocking API since the rest client is async.
    blocking::{Client, Request},
    StatusCode,
    Url,
};
use serde::Deserialize;

pub use timeline_client::TimelineClient;
mod timeline_client {
    //! The Visual Crossing timeline API client.

    use super::*;
    use std::cell::RefCell;
    use std::fmt::Formatter;

    #[derive(Debug)]
    /// The current timeline client request location and client handle.
    struct ActiveRequest {
        /// The location associated with the request.
        location: Location,
        /// The Rest client handle.
        client_handle: RestClientHandle,
    }

    /// The Visual Crossing timeline API Rest client. The client can only run 1 request at a time. A
    /// debug assertion is thrown if [execute](HistoryClient::execute()) is called with a request pending.
    ///
    pub struct TimelineClient {
        /// The Rest async request runner.
        rest_client: RestClient,
        /// The Visual Crossing base URL.
        url: Url,
        /// The Visual Crossing API key.
        api_key: String,
        /// The currently active request.
        active_request: RefCell<Option<ActiveRequest>>,
    }
    impl std::fmt::Debug for TimelineClient {
        /// Show all the attributes except the API client and API key.
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TimelineClient")
                .field("url", &self.url)
                .field("active_request", &self.active_request)
                .finish()
        }
    }
    impl TimelineClient {
        /// Creates a new instance of the HTTP client metadata.
        ///
        /// # Arguments
        ///
        /// * `config` is the weather data configuration.
        ///
        pub fn new(config: &Config) -> Result<Self> {
            let endpoint = if config.visual_crossing.endpoint.ends_with("/") {
                config.visual_crossing.endpoint.clone()
            } else {
                format!("{}/", config.visual_crossing.endpoint)
            };
            match Url::parse(&endpoint) {
                Err(err) => {
                    let reason = format!("Error parsing URL='{}' ({})", endpoint, err);
                    Err(Error::from(reason))
                }
                Ok(url) => match Client::builder().build() {
                    Err(error) => Err(Error::from(format!("Error creating history client ({})", error))),
                    Ok(client) => Ok(Self {
                        rest_client: RestClient::new(client),
                        url,
                        api_key: config.visual_crossing.api_key.clone(),
                        active_request: Default::default(),
                    }),
                },
            }
        }
        /// Creates the Visual Crossing timeline URL to query weather history.
        ///
        /// # Arguments
        ///
        /// * `latitude` is the location latitude.
        /// * `longitude` is the location longitude.
        /// * `date_range` identifies the history dates of interest.
        ///
        fn create_request(&self, location: &Location, date_range: &DateRange) -> Result<Request> {
            // add the location
            let lat_long = format!("{},{}", location.latitude, location.longitude);
            match self.url.join(&lat_long) {
                Err(err) => {
                    Err(Error::from(format!("URL Error adding {} latitude/longitude ({})", location.name, err)))
                }
                Ok(mut url) => {
                    // add in the date range
                    let (from, to) = date_range.as_iso8601();
                    if date_range.is_one_day() {
                        url.path_segments_mut().unwrap().push(&from);
                    } else {
                        url.path_segments_mut().unwrap().push(&from).push(&to);
                    }
                    // add the query parameters
                    let builder = self.rest_client.get(url).query(&[
                        ("unitGroup", "us"),
                        ("include", "days"),
                        ("key", &self.api_key),
                    ]);
                    // build the request
                    match builder.build() {
                        Ok(request) => Ok(request),
                        Err(err) => {
                            let reason = format!("Error building {} history request ({})", location.name, err);
                            Err(Error::from(reason))
                        }
                    }
                }
            }
        }
    }
    impl HistoryClient for TimelineClient {
        /// Use the Visual Crossing timeline API to get history for a location.
        ///
        /// # Arguments
        ///
        /// * `location` is whose history will be queried.
        /// * `date_range` is the history dates to query.
        ///
        fn execute(&self, location: &Location, date_range: &DateRange) -> Result<()> {
            let is_active_request = self.active_request.borrow().is_some();
            match is_active_request {
                true => Err(Error::from("A request already in active."))?,
                false => {
                    let request = self.create_request(location, date_range)?;
                    let client_handle = self.rest_client.execute(request);
                    self.active_request.borrow_mut().replace(ActiveRequest {
                        location: location.clone(),
                        client_handle,
                    });
                    Ok(())
                }
            }
        }
        /// Query if the request has finished or return an error if there is no active request. `Ok(true)`
        /// guarantees the response is available.
        ///
        fn poll(&self) -> Result<bool> {
            match self.active_request.borrow().as_ref() {
                Some(active_request) => Ok(active_request.client_handle.is_finished()),
                None => Err(Error::from("There is no active request available.")),
            }
        }
        /// Get the result by blocking until the request finishes.
        ///
        fn get(&self) -> Result<DailyHistories> {
            match self.active_request.borrow_mut().take() {
                // None => ControlFlow::Break(Err(Error::from("There is no active request."))),
                None => Err(Error::from("There is no active request.")),
                Some(active_request) => match active_request.client_handle.get() {
                    RestClientResult::Body(body) => map_body(active_request.location, body),
                    client_result => map_client_error(&active_request.location, client_result),
                },
            }
        }
    }

    /// Convert the response body into the daily histories.
    ///
    /// # Arguments
    ///
    /// - `location` is the location associated with the response.
    /// - `body` is the raw `JSON` document.
    fn map_body(location: Location, body: Vec<u8>) -> Result<DailyHistories> {
        match serde_json::from_slice::<TimelineDays>(&body[..]) {
            Ok(timeline_days) => Ok(timeline_days.into_daily_histories(&location)),
            Err(err) => Err(Error::from(format!("Error with response body document ({})", err))),
        }
    }

    /// Convert the Rest client error result into an appropriate message.
    ///
    /// # Arguments
    ///
    /// - `location` is the location associated with the response.
    /// - `client_result` is the Rest client result.
    ///
    fn map_client_error(location: &Location, client_result: RestClientResult) -> Result<DailyHistories> {
        use RestClientResult::*;
        let what_happened = match client_result {
            ClientPanic(msg) => format!("Add history for {} panicked ({})", location.name, msg),
            ExecuteError(msg) => format!("Add history for {} did not run ({}).", location.name, msg),
            ResponseError(msg) => format!("Add history for {} response error ({})", location.name, msg),
            HttpStatusCode(code) => {
                let status_code = StatusCode::from_u16(code).unwrap();
                debug_assert!(status_code != StatusCode::OK, "HTTP status is Ok\n{:#?}", location);
                match status_code {
                    StatusCode::TOO_MANY_REQUESTS => "Too many requests today.".to_string(),
                    StatusCode::UNAUTHORIZED => "API key was not accepted.".to_string(),
                    StatusCode::NOT_FOUND => format!(
                        "History not found for '{}' ({}/{}).",
                        location.name, location.latitude, location.longitude
                    ),
                    _ => format!("HTTP error {} ({}).", status_code.as_u16(), status_code.as_str()),
                }
            }
            _ => unreachable!("RestClientResult is not an error"),
        };
        Err(Error::from(what_happened))
    }
}

use timeline_response::TimelineDays;
mod timeline_response {
    //! The Visual Crossing timeline response.

    use super::*;

    /// Defines the fields of interest from the Visual Crossing weather data response.
    #[allow(non_snake_case)]
    #[derive(Debug, Deserialize)]
    struct TimelineDay {
        /// The date associated with the history.
        datetime: String,
        /// The high temperature.
        tempmax: Option<f64>,
        /// The low temperature.
        tempmin: Option<f64>,
        /// The mean temperature.
        temp: Option<f64>,
        /// The dew point.
        dew: Option<f64>,
        /// The humidity.
        humidity: Option<f64>,
        /// The amount of rain.
        precip: Option<f64>,
        /// The chance of rain.
        precipprob: Option<f64>,
        /// The type  of rain (this be null if it's not rainy day).
        preciptype: Option<Vec<String>>,
        /// The highest wind speed recorded.
        windgust: Option<f64>,
        /// The wind speed.
        windspeed: Option<f64>,
        /// The wind direction in degrees.
        winddir: Option<f64>,
        /// The barometric pressure in millibars.
        pressure: Option<f64>,
        /// The percent of sky covered by clouds.
        cloudcover: Option<f64>,
        /// The visibility distance.
        visibility: Option<f64>,
        /// The level of ultraviolet exposure.
        uvindex: Option<f64>,
        /// The time when the sun rises.
        sunriseEpoch: Option<i64>,
        /// The time when the sun sets.
        sunsetEpoch: Option<i64>,
        /// The moons phase.
        moonphase: Option<f64>,
        /// The description of weather for the day.
        description: Option<String>,
    }
    impl TimelineDay {
        /// Convert the visual crossing timeline day into [History].
        ///
        /// # Arguments
        ///
        /// * `alias` is the location alias name.
        ///
        fn into_history(self, alias: &str) -> History {
            History {
                alias: alias.to_string(),
                date: toolslib::date_time::parse_date(&self.datetime).map_or(Default::default(), |d| d),
                temperature_high: self.tempmax,
                temperature_low: self.tempmin,
                temperature_mean: self.temp,
                dew_point: self.dew,
                humidity: self.humidity.map_or(Default::default(), |h| Some(h / 100.0)),
                // there % scale seems to b 0.0 to 100.0
                precipitation_chance: self.precipprob.map_or(Default::default(), |p| Some(p / 100.0)),
                precipitation_type: self.preciptype.map_or(Default::default(), |t| Some(t.join(" "))),
                precipitation_amount: self.precip,
                wind_speed: self.windspeed,
                wind_gust: self.windgust,
                wind_direction: self.winddir.map_or(Default::default(), |d| Some(d.round() as i64)),
                cloud_cover: self.cloudcover.map_or(Default::default(), |c| Some(c / 100.0)),
                pressure: self.pressure,
                uv_index: self.uvindex,
                sunrise: self
                    .sunriseEpoch
                    .map_or(None, |ts| DateTime::from_timestamp(ts, 0))
                    .map_or(None, |dt| Some(dt.naive_utc())),
                sunset: self
                    .sunsetEpoch
                    .map_or(None, |ts| DateTime::from_timestamp(ts, 0))
                    .map_or(None, |dt| Some(dt.naive_utc())),
                moon_phase: self.moonphase,
                visibility: self.visibility,
                description: self.description,
            }
        }
    }

    /// The fields of interest from the Visual Crossing response.
    #[derive(Debug, Deserialize)]
    pub struct TimelineDays {
        /// The weather history days corresponding to the request dates.
        days: Vec<TimelineDay>,
    }
    impl TimelineDays {
        /// Convert the timeline days into daily histories.
        ///
        /// # Arguments
        ///
        /// - `location` is the location associated with the daily histories.
        ///
        pub fn into_daily_histories(self, location: &Location) -> DailyHistories {
            DailyHistories {
                location: location.clone(),
                histories: self
                    .days
                    .into_iter()
                    .map(|timeline_day| timeline_day.into_history(&location.alias))
                    .collect(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use chrono::NaiveDate;

        #[test]
        fn daily_histories() {
            let response = include_str!("response.json");
            let location = Location {
                city: "city".to_string(),
                state_id: "abrev_state".to_string(),
                state: "state".to_string(),
                name: "name".to_string(),
                alias: "alias".to_string(),
                longitude: "-111".to_string(),
                latitude: "47".to_string(),
                tz: "America/Denver".to_string(),
            };
            let timeline_days = serde_json::from_slice::<TimelineDays>(response.as_bytes()).unwrap();
            let daily_histories = timeline_days.into_daily_histories(&location);
            assert_eq!(daily_histories.location.name, location.name);
            assert_eq!(daily_histories.location.alias, location.alias);
            assert_eq!(daily_histories.location.longitude, location.longitude);
            assert_eq!(daily_histories.location.latitude, location.latitude);
            assert_eq!(daily_histories.location.tz, location.tz);
            assert_eq!(daily_histories.histories.len(), 15);
            for day in 0..15 {
                let expected_date = NaiveDate::from_ymd_opt(2024, 3, 1 + day).unwrap();
                let history = daily_histories.histories.get(day as usize).unwrap();
                assert_eq!(history.date, expected_date);
                assert_eq!(history.alias.as_str(), location.alias);
            }
        }
    }
}
