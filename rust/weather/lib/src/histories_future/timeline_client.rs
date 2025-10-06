/// This module contains the client that manages calling the Visual Crossing timeline Rest service.
///
use crate::histories_future::{HistoriesResult, HistoryOutcome};
use crate::{
    prelude::{DateRange, History, Location},
    Result,
};
use reqwest::{
    blocking::{Client, Response},
    StatusCode, Url,
};
use std::sync::Arc;

/// Creates a weather data library error.
macro_rules! err {
    ($reason:expr) => {
        Err(crate::Error::from($reason))
    };
}

/// The client that manages calling the Visual Crossing timeline Rest service.
pub struct TimelineClient {
    /// The location historical weather data is being collection for.
    location: Location,
    /// The URL for the Visual Crossing timeline Rest service.
    url: Url,
    /// The result of getting a response from the Rest service.
    outcome: Arc<HistoryOutcome>,
}
impl TimelineClient {
    /// Creates a new instance of the Visual Crossing timeline manager.
    ///
    /// # Arguments
    ///
    /// * `location` identifies where historical weather data is being collected.
    /// * `dates` provides the timeframe for new historical weather data.
    /// * `endpoint` provides the location of the Visual Crossing timeline Rest service.
    /// * `outcome` is the result of interacting with the timeline Rest service.
    ///
    pub fn new(location: Location, dates: DateRange, endpoint: &str, outcome: Arc<HistoryOutcome>) -> Result<Self> {
        let url = match Url::parse(&endpoint) {
            Err(error) => err!(format!("Error parsing '{endpoint}': {error}"))?,
            Ok(mut url) => {
                match url.path_segments_mut() {
                    Err(_) => err!(format!("'{endpoint}' cannot be a base URL."))?,
                    Ok(mut path_segments) => {
                        path_segments.push(&format!("{},{}", location.latitude, location.longitude));
                        let (start, end) = dates.as_iso8601();
                        path_segments.push(&start);
                        if !dates.is_one_day() {
                            path_segments.push(&end);
                        }
                    }
                }
                url
            }
        };
        Ok(Self { location, url, outcome })
    }
    /// Creates the Rest client and calls the Visual Crossing timeline service. The `outcome` attribute
    /// will be updated with the result of calling the Rest service.
    ///
    /// @Arguments
    ///
    /// * `api_key` is the Visual Crossing timeline API key.
    ///
    pub fn execute(&self, api_key: String) {
        let client = Client::new();
        let request_builder =
            client.get(self.url.clone()).query(&[("unitGroup", "us"), ("include", "days"), ("key", &api_key)]);
        match request_builder.build() {
            Err(error) => self.outcome.set(HistoriesResult::Error(format!("Error building request: {error}"))),
            Ok(request) => match client.execute(request) {
                Err(error) => self.outcome.set(HistoriesResult::Error(format!("Network error: {error}"))),
                Ok(response) => self.map_response(response),
            },
        };
    }
    /// Convert the timeline response into a collection of [DailyHistories].
    ///
    /// * `response` is what the timeline service returned from the Rest call.
    ///
    fn map_response(&self, response: Response) {
        match response.status() {
            // get the response contents
            StatusCode::OK => match response.bytes() {
                Err(error) => {
                    self.outcome.set(HistoriesResult::Error(format!("Failed to get timeline content: {error}")))
                }
                Ok(bytes) => match timeline_response::map_body(&self.location.alias, bytes.into()) {
                    Ok(histories) => self.outcome.set(HistoriesResult::Histories(histories)),
                    Err(error) => self.outcome.set(HistoriesResult::Error(error.to_string())),
                },
            },
            // map the HTTP error
            status_code => {
                let error = match status_code {
                    StatusCode::TOO_MANY_REQUESTS => String::from("Too many requests today."),
                    StatusCode::UNAUTHORIZED => String::from("API key was not accepted."),
                    StatusCode::NOT_FOUND => format!(
                        "History not found for {} at {}/{}.",
                        self.location.name, self.location.latitude, self.location.longitude
                    ),
                    _ => format!(
                        "HTTP error {}: {}.",
                        status_code.as_str(),
                        status_code.canonical_reason().unwrap_or("???")
                    ),
                };
                self.outcome.set(HistoriesResult::Error(error))
            }
        }
    }
}

mod timeline_response {
    //! The Visual Crossing timeline response.

    use super::*;
    use chrono::DateTime;
    use serde::Deserialize;

    /// Convert the response body into the daily histories.
    ///
    /// # Arguments
    ///
    /// - `location` is the location associated with the response.
    /// - `body` is the raw `JSON` document.
    ///
    pub fn map_body(alias: &str, body: Vec<u8>) -> Result<Vec<History>> {
        match serde_json::from_slice::<TimelineDays>(&body[..]) {
            Ok(timeline_days) => Ok(timeline_days.into_histories(alias)),
            Err(error) => err!(format!("Error parsing timeline document: {error}")),
        }
    }

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
        /// - `alias` is the location alias associated with the daily histories.
        ///
        pub fn into_histories(self, alias: &str) -> Vec<History> {
            self.days.into_iter().map(|day| day.into_history(alias)).collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use chrono::NaiveDate;

        #[test]
        fn daily_histories() {
            let response = include_str!("timeline_response.json");
            let timeline_days = serde_json::from_slice::<TimelineDays>(response.as_bytes()).unwrap();
            let alias = "alias";
            let histories = timeline_days.into_histories(alias);
            for day in 0..15 {
                let expected_date = NaiveDate::from_ymd_opt(2024, 3, 1 + day).unwrap();
                // let history = daily_histories.histories.get(day as usize).unwrap();
                let history = histories.get(day as usize).unwrap();
                assert_eq!(history.date, expected_date);
                assert_eq!(history.alias.as_str(), alias);
            }
        }
    }
}
