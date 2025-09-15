//! The weather data history reports.
//!
use super::{csv_to_string, csv_write_record, json_to_string, text_title_separator};
use serde_json::{json, Map, Value};
use toolslib::{header, layout, report::ReportSheet};

use chrono::prelude::*;
use chrono_tz::*;
use weather_lib::prelude::DailyHistories;

/// The report content selection categories.
#[derive(Debug, Default)]
pub struct ReportSelector {
    /// Include temperature related history data.
    pub temperatures: bool,
    /// Include precipitation related history data.
    pub precipitation: bool,
    /// Include the weather conditions related history data.
    pub conditions: bool,
    /// Include summary information for the history data.
    pub summary: bool,
}

fn sanitize_report_selector(report_selector: &mut ReportSelector) {
    if !(report_selector.precipitation || report_selector.conditions || report_selector.summary) {
        // temperatures is the default
        report_selector.temperatures = true;
    }
}

pub mod text {
    //! The report history text based reporting implementation.
    //!
    use super::*;
    use std::fmt::Write;
    use toolslib::{
        date_time::{fmt_date, get_tz_ts},
        fmt::fmt_float,
    };

    const DEFAULT_DATE_FORMAT: &'static str = "%Y-%m-%d";

    /// The text based history report.
    ///
    #[derive(Debug)]
    pub struct Report {
        /// The report content selection.
        report_selector: ReportSelector,
        /// Add a separator between the headers and history data.
        title_separator: bool,
        /// Allow the dates to have a custom format
        date_format: Option<String>,
    }
    impl Report {
        /// Create a new instance of the text based history report.
        ///
        /// # Arguments
        ///
        /// - `report_selection` controls the contents of the report.
        ///
        pub fn new(mut report_selector: ReportSelector) -> Self {
            sanitize_report_selector(&mut report_selector);
            Self { report_selector, title_separator: false, date_format: None }
        }

        /// Add a separator between header rows and report text rows.
        ///
        pub fn with_title_separator(mut self) -> Self {
            self.title_separator = true;
            self
        }

        /// Use a custom date format for report dates.
        ///
        /// # Arguments
        ///
        /// - `date_format` is the `chrono` date format string.
        ///
        pub fn with_date_format(mut self, date_format: &str) -> Self {
            let date_format = date_format.to_string();
            let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
            // write will error if the format is bad
            let mut formatted_epoch = String::new();
            match write!(formatted_epoch, "{}", epoch.format(&date_format)) {
                Ok(_) => {
                    self.date_format.replace(date_format);
                }
                Err(_) => {
                    // right now formats are all hard coded so it's a dev problem
                    debug_assert!(false, "Bad date format '{}'!!!", date_format);
                }
            }
            self
        }

        /// Generates the report history text based report.
        ///
        /// # Arguments
        ///
        /// * `daily_histories` is the locations_win weather history that will be reported.
        ///
        pub fn generate(&self, daily_histories: DailyHistories) -> ReportSheet {
            let mut layouts = vec![layout!(^)];
            macro_rules! layouts {
                ($layouts:expr) => {
                    layouts.append(&mut $layouts);
                };
            }
            let mut header1 = vec![header!("")];
            macro_rules! header1 {
                ($columns:expr) => {
                    header1.append(&mut $columns);
                };
            }
            let mut header2 = vec![header!("Date")];
            macro_rules! header2 {
                ($columns:expr) => {
                    header2.append(&mut $columns);
                };
            }
            if self.report_selector.temperatures {
                layouts!(vec![layout!(^), layout!(^), layout!(^), layout!(^)]);
                header1!(vec![header!(+ "-"), header!("Temperature"), header!(+ "-"), header!("Dew")]);
                header2!(vec![header!("High"), header!("Low"), header!("Mean"), header!("Point")]);
            }
            if self.report_selector.precipitation {
                layouts!(vec![layout!(^), layout!(^), layout!(^), layout!(^), layout!(^)]);
                header1!(vec![header!("Cloud"), header!(""), header!(+ "-"), header!("Precipitation"), header!(+ "-")]);
                header2!(vec![
                    header!("Cover"),
                    header!("Humidity"),
                    header!("Chance"),
                    header!("Amount"),
                    header!("Type")
                ]);
            }
            if self.report_selector.conditions {
                layouts!(vec![layout!(>), layout!(>), layout!(^), layout!(^), layout!(^)]);
                header1!(vec![header!(+ "-"), header!("Wind"), header!(+ "-"), header!(""), header!("UV")]);
                header2!(vec![
                    header!("Speed"),
                    header!("Gust"),
                    header!("Bearing"),
                    header!("Pressure"),
                    header!("Index")
                ]);
            }
            if self.report_selector.summary {
                layouts!(vec![layout!(^), layout!(^), layout!(^), layout!(<)]);
                header1!(vec![header!(""), header!(""), header!("Moon"), header!("")]);
                header2!(vec![header!("Sunrise"), header!("Sunset"), header!("Phase"), header!("Summary")]);
            }
            let columns = layouts.len();
            let mut report = ReportSheet::new(layouts);
            report.add_row(header1);
            report.add_row(header2);
            if self.title_separator {
                report.add_row(text_title_separator!(columns));
            }
            let tz: Tz = daily_histories.location.tz.parse().unwrap();
            let date_format = self.date_format.as_ref().map_or(DEFAULT_DATE_FORMAT, |format| format.as_str());
            for history in daily_histories.histories {
                let mut row = Vec::with_capacity(columns);
                row.push(toolslib::text!(fmt_date(&history.date, date_format)));
                if self.report_selector.temperatures {
                    row.push(toolslib::text!(fmt_temperature(&history.temperature_high)));
                    row.push(toolslib::text!(fmt_temperature(&history.temperature_low)));
                    row.push(toolslib::text!(fmt_temperature(&history.temperature_mean)));
                    row.push(toolslib::text!(fmt_temperature(&history.dew_point)));
                }
                if self.report_selector.precipitation {
                    row.push(toolslib::text!(fmt_percent(&history.cloud_cover)));
                    row.push(toolslib::text!(fmt_percent(&history.humidity)));
                    row.push(toolslib::text!(fmt_percent(&history.precipitation_chance)));
                    row.push(toolslib::text!(fmt_float(&history.precipitation_amount, 2)));
                    row.push(toolslib::text!(history
                        .precipitation_type
                        .as_ref()
                        .map_or(Default::default(), |t| t.as_str())));
                }
                if self.report_selector.conditions {
                    row.push(toolslib::text!(fmt_float(&history.wind_speed, 1)));
                    row.push(toolslib::text!(fmt_float(&history.wind_gust, 1)));
                    row.push(toolslib::text!(fmt_wind_bearing(&history.wind_direction)));
                    row.push(toolslib::text!(fmt_float(&history.pressure, 1)));
                    row.push(toolslib::text!(fmt_uv_index(&history.uv_index)));
                }
                // if self.summary {
                if self.report_selector.summary {
                    row.push(toolslib::text!(fmt_hhmm(&history.sunrise, &tz)));
                    row.push(toolslib::text!(fmt_hhmm(&history.sunset, &tz)));
                    row.push(toolslib::text!(fmt_moon_phase(&history.moon_phase)));
                    row.push(toolslib::text!(history.description.as_ref().map_or(Default::default(), |s| s.as_str())));
                }
                report.add_row(row);
            }
            report
        }
    }

    /// Returns a compass bearing as a human readable direction.
    ///
    /// The four cardinal points on a compass are subdivided into a finer grained
    /// direction strings as shown below:
    ///
    /// ```
    /// N NNE NE ENE
    /// E ESE SE SSE
    /// S SSW SW WSW
    /// W WNW NW NNW
    /// ```
    ///
    /// There is a window around the absolute direction to determine the bearing string.
    /// As an example any bearing between 348.75 degrees and 11.25 degrees will be returned
    /// as a `N` bearing string.
    ///
    /// If the option is `None` an empty string will be returned.
    ///
    /// # Arguments
    ///
    /// * `bearing_option` - the bearing that will be converter to a string.
    ///
    fn fmt_wind_bearing(bearing_option: &Option<i64>) -> &'static str {
        if let Some(bearing) = bearing_option {
            static BEARINGS: [&'static str; 16] =
                ["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"];
            let index = ((*bearing as f64 / 22.5) + 0.5) as usize % 16;
            BEARINGS[index]
        } else {
            Default::default()
        }
    }

    /// Returns a percentage as a string.
    ///
    /// The percentage is rounded to an integer value and contains a *%* trailing the value.
    /// The following table provides sample output.
    ///
    /// | Value | Result |
    /// | ---: | ---: |
    /// | 0.0 | 0% |
    /// | 25.4 | 25% |
    /// | 99.5 | 100% |
    ///
    /// If the option is `None` an empty string will be returned.
    ///
    fn fmt_percent(option: &Option<f64>) -> String {
        match option {
            Some(value) => format!("{:>3}%", ((value * 100.0) + 0.5) as i64),
            None => Default::default(),
        }
    }

    /// Returns a temperature as a string.
    ///
    /// The temperature is rounded to the nearest 1/10 degree.
    ///
    /// If the option is `None` an empty string will be returned.
    ///
    #[inline]
    fn fmt_temperature(t: &Option<f64>) -> String {
        match t {
            Some(temperature) => format!("{:>-5.1}", temperature),
            None => Default::default(),
        }
    }

    /// Returns a timestamp as hours and minutes string.
    ///
    /// The string will follow the form `hh:mm` where:
    ///
    /// * `hh` is the 2 digit hour (0-23)
    /// * `mm` is the hour minutes (0-59)
    ///
    /// If the option is `None` an empty string will be returned.
    ///
    #[inline]
    fn fmt_hhmm(date_time: &Option<NaiveDateTime>, tz: &Tz) -> String {
        date_time.map_or(Default::default(), |dt| get_tz_ts(dt.and_utc().timestamp(), tz).format("%H:%M").to_string())
    }

    /// Returns a UV index as a human readable string.
    ///
    /// The possible UV index strings are:
    ///
    /// | UV Index | Description |
    /// | :----: | :----: |
    /// | 1-2 | low |
    /// | 3-5 | moderate |
    /// | 6-7 | high |
    /// | 8-10 | very high |
    /// | 11+ | extreme |
    ///
    /// If the option is `None` or the value 0, an empty string will be returned.
    ///
    fn fmt_uv_index(option: &Option<f64>) -> &'static str {
        let mut uv_index = "";
        if let Some(value) = option {
            let value = value.round() as i64;
            if value > 0 {
                uv_index = match value {
                    1 | 2 => "low",
                    3 | 4 | 5 => "moderate",
                    6 | 7 => "high",
                    8 | 9 | 10 => "very high",
                    _ => "extreme",
                };
            }
        }
        uv_index
    }

    /// Returns moon phase as a human readable string.
    ///
    /// The possible moon phase indicators are:
    ///
    /// | Moon Phase | Description |
    /// | :----: | :----: |
    /// | 0 | new moon |
    /// | 0-0.25 | waxing crescent |
    /// | 0.25 | first quarter |
    /// | 0.25-0.5 | waxing gibbous |
    /// | 0.5 | full moon |
    /// | 0.5-0.75 | waning gibbous |
    /// | 0.75 | last quarter |
    /// | 0.75-1.0 | waning crescent |
    ///
    /// If the option is `None` an empty string will be returned.
    ///
    fn fmt_moon_phase(option: &Option<f64>) -> &'static str {
        let mut moon_phase = "";
        if let Some(value) = option {
            let phase = *value;
            moon_phase = if phase >= 0.0 && phase <= 0.01 {
                "new moon"
            } else if phase > 0.01 && phase < 0.24 {
                "waxing crescent"
            } else if phase >= 0.24 && phase <= 0.26 {
                "first quarter"
            } else if phase > 0.26 && phase < 0.49 {
                "waxing gibbous"
            } else if phase >= 0.49 && phase <= 0.51 {
                "full moon"
            } else if phase > 0.51 && phase < 0.74 {
                "waning gibbous"
            } else if phase >= 0.74 && phase <= 0.76 {
                "last quarter"
            } else if phase > 0.76 && phase <= 1.0 {
                "waning crescent"
            } else {
                "unknown"
            };
        }
        moon_phase
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use toolslib::date_time::{get_date, get_time};

        #[test]
        fn hhmm() {
            let tz: Tz = "America/Phoenix".parse().unwrap();
            assert_eq!(fmt_hhmm(&None, &tz), "");
            let date_time = NaiveDateTime::new(get_date(2023, 9, 23), get_time(22, 22, 22));
            assert_eq!(fmt_hhmm(&Some(date_time), &tz), "15:22");
        }

        #[test]
        fn wind_bearing() {
            for bearing in 0..=11 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "N");
            }
            for bearing in 12..=33 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "NNE");
            }
            for bearing in 34..=56 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "NE");
            }
            for bearing in 57..=78 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "ENE");
            }
            for bearing in 79..=101 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "E");
            }
            for bearing in 102..=123 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "ESE");
            }
            for bearing in 124..=146 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "SE");
            }
            for bearing in 147..=168 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "SSE");
            }
            for bearing in 169..=191 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "S");
            }
            for bearing in 192..=213 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "SSW");
            }
            for bearing in 214..=236 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "SW");
            }
            for bearing in 237..=258 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "WSW");
            }
            for bearing in 259..=281 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "W");
            }
            for bearing in 282..=303 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "WNW");
            }
            for bearing in 304..=326 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "NW");
            }
            for bearing in 327..=348 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "NNW");
            }
            for bearing in 349..=361 {
                assert_eq!(fmt_wind_bearing(&Some(bearing)), "N");
            }
        }

        #[test]
        fn percent() {
            assert_eq!(fmt_percent(&None), "");
            assert_eq!(fmt_percent(&Some(0.0)), "  0%");
            assert_eq!(fmt_percent(&Some(0.1049)), " 10%");
            assert_eq!(fmt_percent(&Some(0.995)), "100%");
        }

        #[test]
        fn temperature() {
            assert_eq!(fmt_temperature(&None), "");
            assert_eq!(fmt_temperature(&Some(50.94)), " 50.9");
            assert_eq!(fmt_temperature(&Some(50.95)), " 51.0");
            assert_eq!(fmt_temperature(&Some(99.9)), " 99.9");
            assert_eq!(fmt_temperature(&Some(-29.9)), "-29.9");
        }

        #[test]
        fn uv_index() {
            assert_eq!(fmt_uv_index(&None), "");
            assert_eq!(fmt_uv_index(&Some(0.0)), "");
            assert_eq!(fmt_uv_index(&Some(1.0)), "low");
            assert_eq!(fmt_uv_index(&Some(2.0)), "low");
            assert_eq!(fmt_uv_index(&Some(3.0)), "moderate");
            assert_eq!(fmt_uv_index(&Some(4.0)), "moderate");
            assert_eq!(fmt_uv_index(&Some(5.0)), "moderate");
            assert_eq!(fmt_uv_index(&Some(6.0)), "high");
            assert_eq!(fmt_uv_index(&Some(7.0)), "high");
            assert_eq!(fmt_uv_index(&Some(8.0)), "very high");
            assert_eq!(fmt_uv_index(&Some(9.0)), "very high");
            assert_eq!(fmt_uv_index(&Some(10.0)), "very high");
            assert_eq!(fmt_uv_index(&Some(11.0)), "extreme");
            assert_eq!(fmt_uv_index(&Some(12.0)), "extreme");
        }

        #[test]
        fn moon_phase() {
            assert_eq!(fmt_moon_phase(&None), "");
            assert_eq!(fmt_moon_phase(&Some(0.0)), "new moon");
            assert_eq!(fmt_moon_phase(&Some(0.01)), "new moon");
            assert_eq!(fmt_moon_phase(&Some(0.011)), "waxing crescent");
            assert_eq!(fmt_moon_phase(&Some(0.239)), "waxing crescent");
            assert_eq!(fmt_moon_phase(&Some(0.24)), "first quarter");
            assert_eq!(fmt_moon_phase(&Some(0.26)), "first quarter");
            assert_eq!(fmt_moon_phase(&Some(0.261)), "waxing gibbous");
            assert_eq!(fmt_moon_phase(&Some(0.489)), "waxing gibbous");
            assert_eq!(fmt_moon_phase(&Some(0.49)), "full moon");
            assert_eq!(fmt_moon_phase(&Some(0.51)), "full moon");
            assert_eq!(fmt_moon_phase(&Some(0.511)), "waning gibbous");
            assert_eq!(fmt_moon_phase(&Some(0.739)), "waning gibbous");
            assert_eq!(fmt_moon_phase(&Some(0.74)), "last quarter");
            assert_eq!(fmt_moon_phase(&Some(0.76)), "last quarter");
            assert_eq!(fmt_moon_phase(&Some(0.761)), "waning crescent");
            assert_eq!(fmt_moon_phase(&Some(1.0)), "waning crescent");
            assert_eq!(fmt_moon_phase(&Some(1.001)), "unknown");
        }
    }
}

pub mod json {
    /// The report history JSON based reporting implementation.
    ///
    use super::*;
    use toolslib::date_time::{get_tz_ts, isodate};

    /// The `JSON` based weather history report.
    ///
    #[derive(Debug)]
    pub struct Report {
        /// Controls the content of the weather history report.
        report_selector: ReportSelector,
        /// Controls if the resulting document will be pretty printed of not.
        pretty: bool,
    }
    impl Report {
        /// Create a new instance of the `JSON` based weather history report.
        ///
        /// # Arguments
        ///
        /// - `report_selection` controls the contents of the report.
        ///
        pub fn new(mut report_selector: ReportSelector) -> Self {
            sanitize_report_selector(&mut report_selector);
            Self { report_selector, pretty: false }
        }
        /// Create a new instance of the `JSON` based weather history report that produces pretty printed documents.
        ///
        /// # Arguments
        ///
        /// - `report_selection` controls the contents of the report.
        ///
        pub fn pretty_printed(mut report_selector: ReportSelector) -> Self {
            sanitize_report_selector(&mut report_selector);
            Self { report_selector, pretty: true }
        }
        /// Generates the report history JSON based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `daily_histories` is the locations_win weather history that will be reported.
        ///
        pub fn generate(&self, daily_histories: DailyHistories) -> String {
            let mut values: Vec<Map<String, Value>> = vec![];
            let tz: Tz = daily_histories.location.tz.parse().unwrap();
            for history in daily_histories.histories {
                let mut value = Map::new();
                let mut add = |key: &str, v: Value| value.insert(key.to_string(), v);
                add("date", json!(isodate(&history.date)));
                if self.report_selector.temperatures {
                    add("temperatureHigh", float_value(&history.temperature_high));
                    add("temperatureLow", float_value(&history.temperature_low));
                    add("temperatureMean", float_value(&history.temperature_mean));
                    add("dewPoint", float_value(&history.dew_point));
                }
                if self.report_selector.precipitation {
                    add("cloudCover", float_value(&history.cloud_cover));
                    add("humidity", float_value(&history.humidity));
                    add("precip", float_value(&history.precipitation_amount));
                    add("precipChance", float_value(&history.precipitation_chance));
                    add("precipType", string_value(&history.precipitation_type));
                }
                if self.report_selector.conditions {
                    add("windSpeed", float_value(&history.wind_speed));
                    add("windGust", float_value(&history.wind_gust));
                    add("windBearing", int_value(&history.wind_direction));
                    add("uvIndex", float_value(&history.uv_index));
                    add("pressure", float_value(&history.pressure));
                }
                if self.report_selector.summary {
                    add("sunrise", datetime_value(&history.sunrise, &tz));
                    add("sunset", datetime_value(&history.sunset, &tz));
                    add("moonPhase", float_value(&history.moon_phase));
                    add("summary", string_value(&history.description));
                }
                values.push(value);
            }
            let json = json!({
                "location": daily_histories.location.name,
                "type": Value::String("daily_history".to_string()),
                "history": json![values],
            });
            json_to_string(json, self.pretty)
        }
    }

    /// Returns a `Value::String(...) ` containing an IETF RFC3339 date timestamp.
    ///
    /// The binary timestamp is converted to a string following the form `YYYY-MM-DDThh:mm:ss+hh:mm`
    /// where:
    ///
    /// * `YYYY` is the 4 digit year
    /// * `MM` is the 2 digit month
    /// * `DD` is the 2 digit day of month
    /// * `hh` is the 2 digit hour of day
    /// * `mm` is the 2 digit minutes within hour
    /// * `ss` is the 2 digit seconds within minute
    /// * `+hh:mm` is the timezone offset. This could be replaced with `Z` however there are no
    /// timezones currently within the UTC zone.
    ///
    /// If option is `None` a `Value::Null` will be returned.
    ///
    /// # Arguments
    ///
    /// * `option` - the timestamp used to create the IETF datetime value.
    /// * `tz` - the timezone associated with the timestamp.
    ///
    fn datetime_value(option: &Option<NaiveDateTime>, tz: &Tz) -> Value {
        match option {
            Some(date_time) => {
                // let dt: DateTime<Tz> = tz.timestamp(*timestamp, 0);
                let dt: DateTime<Tz> = get_tz_ts(date_time.and_utc().timestamp(), tz);
                let iso8601 = dt.to_rfc3339_opts(SecondsFormat::Secs, true);
                json!(iso8601)
            }
            None => Value::Null,
        }
    }

    /// Returns a `Value::String(...)` containing a string value.
    ///
    /// If option is `None` a `Value::Null` will be returned.
    ///
    /// # Arguments
    ///
    /// * `option` - the string that will be encoded as a value.
    ///
    #[inline]
    fn string_value(option: &Option<String>) -> Value {
        match option {
            Some(string) => json!(string),
            None => Value::Null,
        }
    }

    /// Returns a `Value::Number(...)` containing the integer value.
    ///
    /// If option is `None` a `Value::Null` will be returned.
    ///
    /// # Arguments
    ///
    /// * `option` - the integer that will be encoded as a value.
    ///
    #[inline]
    fn int_value(option: &Option<i64>) -> Value {
        match option {
            Some(int) => json!(int),
            None => Value::Null,
        }
    }

    /// Returns a `Value::Number(...)` containing the float value.
    ///
    /// If option is `None` a `Value::Null` will be returned.
    ///
    /// # Arguments
    ///
    /// * `option` - the float that will be encoded as a value.
    ///
    #[inline]
    fn float_value(option: &Option<f64>) -> Value {
        match option {
            Some(float) => json!(float),
            None => Value::Null,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use toolslib::date_time::{get_date, get_time};

        #[test]
        fn datetime() {
            let tz: Tz = "America/Los_Angeles".parse().unwrap();
            assert_eq!(datetime_value(&None, &tz), Value::Null);
            let dt = NaiveDateTime::new(get_date(2023, 9, 23), get_time(23, 23, 23));
            assert_eq!(datetime_value(&Some(dt), &tz), "2023-09-23T16:23:23-07:00".to_string());
        }

        #[test]
        fn strings() {
            assert_eq!(string_value(&None), Value::Null);
            let testcase = "foobar".to_string();
            assert_eq!(string_value(&Some(testcase.clone())), json!(testcase));
        }

        #[test]
        fn numbers() {
            assert_eq!(float_value(&None), Value::Null);
            assert_eq!(float_value(&Some(123.456)), json!(123.456));
            assert_eq!(int_value(&None), Value::Null);
            assert_eq!(int_value(&Some(123456)), json!(123456));
        }
    }
}

pub mod csv {
    /// The report history CSV based reporting implementation.
    ///
    extern crate csv as csv_lib;
    use super::*;
    use toolslib::date_time::{get_tz_ts, isodate};

    /// The `CSV` based weather history report.
    ///
    #[derive(Debug)]
    pub struct Report(
        /// Controls the contents of the weather history report.
        ReportSelector,
    );
    impl Report {
        /// Create a new instance of the `CSV` based weather history report.
        ///
        /// # Arguments
        ///
        /// - `report_selection` controls the contents of the report.
        ///
        pub fn new(mut report_selector: ReportSelector) -> Self {
            sanitize_report_selector(&mut report_selector);
            Self(report_selector)
        }

        /// Generates the list history CSV based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `daily_histories` is the locations_win weather history that will be reported.
        ///
        pub fn generate(&self, daily_histories: DailyHistories) -> String {
            let mut writer = csv_lib::Writer::from_writer(vec![]);
            let mut labels: Vec<&str> = vec!["date"];
            if self.0.temperatures {
                labels.push("temperatureHigh");
                labels.push("temperatureLow");
                labels.push("temperatureMean");
                labels.push("dewPoint");
            }
            if self.0.precipitation {
                labels.push("cloudCover");
                labels.push("humidity");
                labels.push("precip");
                labels.push("precipChance");
                labels.push("precipType");
            }
            if self.0.conditions {
                labels.push("windSpeed");
                labels.push("windGust");
                labels.push("windBearing");
                labels.push("uvIndex");
                labels.push("pressure");
            }
            if self.0.summary {
                labels.push("sunrise");
                labels.push("sunset");
                labels.push("moonPhase");
                labels.push("summary");
            }
            csv_write_record!(writer, &labels);
            let tz: Tz = daily_histories.location.tz.parse().unwrap();
            for daily_history in daily_histories.histories {
                let mut history = vec![isodate(&daily_history.date)];
                if self.0.temperatures {
                    history.push(float_value(&daily_history.temperature_high));
                    history.push(float_value(&daily_history.temperature_low));
                    history.push(float_value(&daily_history.temperature_mean));
                    history.push(float_value(&daily_history.dew_point));
                }
                if self.0.precipitation {
                    history.push(float_value(&daily_history.cloud_cover));
                    history.push(float_value(&daily_history.humidity));
                    history.push(float_value(&daily_history.precipitation_amount));
                    history.push(float_value(&daily_history.precipitation_chance));
                    history.push(string_value(&daily_history.precipitation_type));
                }
                if self.0.conditions {
                    history.push(float_value(&daily_history.wind_speed));
                    history.push(float_value(&daily_history.wind_gust));
                    history.push(int_value(&daily_history.wind_direction));
                    history.push(float_value(&daily_history.uv_index));
                    history.push(float_value(&daily_history.pressure));
                }
                if self.0.summary {
                    history.push(datetime_value(&daily_history.sunrise, &tz));
                    history.push(datetime_value(&daily_history.sunset, &tz));
                    history.push(float_value(&daily_history.moon_phase));
                    history.push(string_value(&daily_history.description));
                }
                csv_write_record!(writer, &history);
            }
            csv_to_string(writer)
        }
    }

    /// Returns an IETF RFC3339 date timestamp string.
    ///
    /// The binary timestamp is converted to a string following the form `YYYY-MM-DDThh:mm:ss+hh:mm`
    /// where:
    ///
    /// * `YYYY` is the 4 digit year
    /// * `MM` is the 2 digit month
    /// * `DD` is the 2 digit day of month
    /// * `hh` is the 2 digit hour of day
    /// * `mm` is the 2 digit minutes within hour
    /// * `ss` is the 2 digit seconds within minute
    /// * `+hh:mm` is the timezone offset. This could be replaced with `Z` however there are no
    /// timezones currently within the UTC zone.
    ///
    /// If option is `None` an empty string will be returned.
    ///
    /// # Arguments
    ///
    /// * `option` - the timestamp used to create the IETF datetime value.
    /// * `tz` - the timezone associated with the timestamp.
    ///
    fn datetime_value(option: &Option<NaiveDateTime>, tz: &Tz) -> String {
        match option {
            Some(date_time) => {
                // let dt: DateTime<Tz> = tz.timestamp(*timestamp, 0);
                let dt: DateTime<Tz> = get_tz_ts(date_time.and_utc().timestamp(), tz);
                dt.to_rfc3339_opts(SecondsFormat::Secs, true)
            }
            None => "".to_string(),
        }
    }

    /// Returns a copy of a string value.
    ///
    /// If option is `None` an empty string will be returned.
    ///
    /// # Arguments
    ///
    /// * `option` - the string that will be copied.
    ///
    #[inline]
    fn string_value(option: &Option<String>) -> String {
        match option {
            Some(string) => string.clone(),
            None => "".to_string(),
        }
    }

    /// Returns an integer value as a string value.
    ///
    /// If option is `None` an empty string will be returned.
    ///
    /// # Arguments
    ///
    /// * `option` - the integer that will be converted to a string.
    ///
    #[inline]
    fn int_value(option: &Option<i64>) -> String {
        match option {
            Some(int) => int.to_string(),
            None => "".to_string(),
        }
    }

    /// Returns a float value as a string value.
    ///
    /// If option is `None` an empty string will be returned.
    ///
    /// # Arguments
    ///
    /// * `option` - the float that will be converted to a string.
    ///
    #[inline]
    fn float_value(option: &Option<f64>) -> String {
        match option {
            Some(float) => float.to_string(),
            None => "".to_string(),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use toolslib::date_time::{get_date, get_time};

        #[test]
        fn datetime() {
            let tz: Tz = "America/Los_Angeles".parse().unwrap();
            assert_eq!(datetime_value(&None, &tz), "".to_string());
            let dt = NaiveDateTime::new(get_date(2023, 9, 23), get_time(23, 23, 23));
            assert_eq!(datetime_value(&Some(dt), &tz), "2023-09-23T16:23:23-07:00".to_string());
        }

        #[test]
        fn strings() {
            assert_eq!(string_value(&None), "".to_string());
            let testcase = "foobar".to_string();
            assert_eq!(string_value(&Some(testcase.clone())), testcase);
        }

        #[test]
        fn numbers() {
            assert_eq!(float_value(&None), "".to_string());
            assert_eq!(float_value(&Some(123.456)), 123.456.to_string());
            assert_eq!(int_value(&None), "".to_string());
            assert_eq!(int_value(&Some(123456)), 123456.to_string());
        }
    }
}
