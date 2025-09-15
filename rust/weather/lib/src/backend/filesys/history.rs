//! Manages serializing and deserializing weather data history JSON documents.
//!
use crate::entities::History;
use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};


/// Create a Locations specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("HistoryDocument {}", format!($($arg)*)))
    }
}

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// This is the structure used to serialize and deserialize [History].
#[derive(Debug, Deserialize, Serialize)]
struct HistoryDocument {
    /// The histories date.
    date: NaiveDate,
    /// The time in seconds (UTC) the sun rises.
    sunrise: Option<i64>,
    /// The time in seconds (UTC) the sun sets.
    sunset: Option<i64>,
    /// The phase of the moon.
    moon: Option<f64>,
    /// The maximum temperature for the date.
    tempmax: Option<f64>,
    /// The minimum temperature for the date.
    tempmin: Option<f64>,
    /// The temperature mean for the date.
    tempmean: Option<f64>,
    /// The dew point.
    dewpoint: Option<f64>,
    /// The chance of rain.
    precipprob: Option<f64>,
    /// The amount of precipitation.
    precip: Option<f64>,
    /// A description of the type of precipitation.
    preciptype: Option<String>,
    /// The humidity.
    humidity: Option<f64>,
    /// The atmospheric pressure in millibars.
    pressure: Option<f64>,
    /// The percent of cloud cover.
    cloud: Option<f64>,
    /// The UV index.
    uv: Option<f64>,
    /// The visibility in miles.
    vis: Option<f64>,
    /// The wind speed in miles per hour.
    wind: Option<f64>,
    /// The maximum wind gust speed in miles per hour.
    windgust: Option<f64>,
    /// The predominant wind speed.
    winddir: Option<i64>,
    /// A summary description of the weather.
    summary: Option<String>,
}
impl HistoryDocument {
    /// Convert the deserialized history to a [History] instance.
    ///
    /// # Arguments
    ///
    /// * `alias` is the location alias name.
    fn to_history(self, alias: &str) -> History {
        History {
            alias: alias.to_string(),
            date: self.date,
            temperature_high: self.tempmax,
            temperature_low: self.tempmin,
            temperature_mean: self.tempmean,
            dew_point: self.dewpoint,
            humidity: self.humidity,
            precipitation_chance: self.precipprob,
            precipitation_type: self.preciptype,
            precipitation_amount: self.precip,
            wind_speed: self.wind,
            wind_gust: self.windgust,
            wind_direction: self.winddir,
            cloud_cover: self.cloud,
            pressure: self.pressure,
            uv_index: self.uv,
            sunrise: self
                .sunrise
                .map_or(None, |ts| DateTime::from_timestamp(ts, 0))
                .map_or(None, |dt| Some(dt.naive_utc())),
            sunset: self
                .sunset
                .map_or(None, |ts| DateTime::from_timestamp(ts, 0))
                .map_or(None, |dt| Some(dt.naive_utc())),
            moon_phase: self.moon,
            visibility: self.vis,
            description: self.summary,
        }
    }
}
impl From<&History> for HistoryDocument {
    /// Convert [History] into the document that can be serialized and deserialized.
    fn from(history: &History) -> Self {
        Self {
            date: history.date,
            sunrise: history.sunrise.map_or(None, |ndt| Some(ndt.and_utc())).map_or(None, |dt| Some(dt.timestamp())),
            sunset: history.sunset.map_or(None, |ndt| Some(ndt.and_utc())).map_or(None, |dt| Some(dt.timestamp())),
            moon: history.moon_phase,
            tempmax: history.temperature_high,
            tempmin: history.temperature_low,
            tempmean: history.temperature_mean,
            dewpoint: history.dew_point,
            precipprob: history.precipitation_chance,
            precip: history.precipitation_amount,
            preciptype: history.precipitation_type.clone(),
            humidity: history.humidity,
            pressure: history.pressure,
            cloud: history.cloud_cover,
            uv: history.uv_index,
            vis: history.visibility,
            wind: history.wind_speed,
            windgust: history.wind_gust,
            winddir: history.wind_direction,
            summary: history.description.clone(),
        }
    }
}

/// Convert [History] into a string.
///
/// # Arguments
///
/// * `history` will be converted to a sequence of bytes.
pub fn to_json(history: &History) -> crate::Result<String> {
    match serde_json::to_string(&HistoryDocument::from(history)) {
        Ok(json) => Ok(json),
        Err(error) => {
            err!("'{}' error serializing history on {}: {:?}", history.alias, history.date, error)
        }
    }
}

/// Convert [History] into a sequence of bytes.
///
/// # Arguments
///
/// * `history` will be converted to a sequence of bytes.
pub fn to_bytes(history: &History) -> crate::Result<Vec<u8>> {
    Ok(to_json(history)?.into_bytes())
}

/// Convert a sequence of bytes into a [History].
///
/// # Arguments
///
/// * `alias` is the locations alias name.
/// * `bytes` will be converted to a [History] instance.
pub fn from_bytes(alias: &str, bytes: &[u8]) -> crate::Result<History> {
    match serde_json::from_slice::<HistoryDocument>(bytes) {
        Ok(history_doc) => Ok(history_doc.to_history(alias)),
        Err(error) => {
            err!("'{}' error deserializing history: {:?}", alias, error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use toolslib::date_time::{get_date, get_time};

    #[test]
    fn json() {
        let alias = "test";
        let history = History {
            alias: alias.to_string(),
            date: get_date(2023, 9, 12),
            temperature_high: Some(77.0),
            temperature_low: Some(56.0),
            temperature_mean: Some(65.8),
            dew_point: Some(60.3),
            humidity: Some(43.0),
            precipitation_chance: Some(8.0),
            precipitation_type: Some("rain".to_string()),
            precipitation_amount: Some(0.1),
            wind_speed: Some(6.0),
            wind_gust: Some(8.0),
            wind_direction: Some(337),
            cloud_cover: Some(7.3),
            pressure: Some(30.05),
            uv_index: Some(5.0),
            sunrise: Some(NaiveDateTime::new(get_date(2023, 9, 12), get_time(13, 45, 0))),
            sunset: Some(NaiveDateTime::new(get_date(2023, 9, 13), get_time(2, 28, 0))),
            moon_phase: Some(0.8),
            visibility: Some(10.0),
            description: Some("Sun and clouds mixed.".to_string()),
        };
        let json = to_bytes(&history).unwrap();
        let testcase = from_bytes(alias, json.as_slice()).unwrap();
        assert_eq!(history.alias, testcase.alias);
        assert_eq!(history.date, testcase.date);
        assert_eq!(history.temperature_high, testcase.temperature_high);
        assert_eq!(history.temperature_low, testcase.temperature_low);
        assert_eq!(history.temperature_mean, testcase.temperature_mean);
        assert_eq!(history.dew_point, testcase.dew_point);
        assert_eq!(history.humidity, testcase.humidity);
        assert_eq!(history.precipitation_chance, testcase.precipitation_chance);
        assert_eq!(history.precipitation_type, testcase.precipitation_type);
        assert_eq!(history.precipitation_amount, testcase.precipitation_amount);
        assert_eq!(history.wind_speed, testcase.wind_speed);
        assert_eq!(history.wind_gust, testcase.wind_gust);
        assert_eq!(history.wind_direction, testcase.wind_direction);
        assert_eq!(history.cloud_cover, testcase.cloud_cover);
        assert_eq!(history.pressure, testcase.pressure);
        assert_eq!(history.uv_index, testcase.uv_index);
        assert_eq!(history.sunrise, testcase.sunrise);
        assert_eq!(history.sunset, testcase.sunset);
        assert_eq!(history.moon_phase, testcase.moon_phase);
        assert_eq!(history.visibility, testcase.visibility);
        assert_eq!(history.description, testcase.description);
    }
}
