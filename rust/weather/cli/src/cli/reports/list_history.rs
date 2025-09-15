//! Generates the weather data location histories report.
//! 
use super::{csv_to_string, csv_write_record, json_to_string, text_title_separator};
use serde_json::{json, Value};
use toolslib::{header, layout, report::ReportSheet};
use weather_lib::prelude::HistoryDates;

pub mod text {
    //! The list history text based reporting implementation.
    //!
    use super::*;
    use chrono::NaiveDate;
    use std::fmt::Write;
    use toolslib::date_time::fmt_date;

    /// The metadata controlling the report appearance.
    #[derive(Debug, Default)]
    pub struct Report {
        /// Controls if a separator row will be added between the report headers and report text.
        title_separator: bool,
        /// Controls the format of printed dates.
        date_format: Option<String>,
    }
    impl Report {
        /// Adds a separator row between the report headers and report text.
        ///
        pub fn with_title_separator(mut self) -> Self {
            self.title_separator = true;
            self
        }
        /// Use a custom date format.
        ///
        /// # Arguments
        ///
        /// - `date_format` describes the format of printed dates.
        ///
        pub fn with_date_format(mut self, date_format: impl ToString) -> Self {
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
        /// Generates the locations_win history text based report.
        ///
        /// # Arguments
        ///
        /// * `location_history_dates` - The list of location and history dates that will be reported.
        ///
        pub fn generate(&self, locations_history_dates: Vec<HistoryDates>) -> ReportSheet {
            let mut report = ReportSheet::new(vec![layout!(<), layout!(<)]);
            report.add_row(vec![header!(^ "Location"), header!(^ "History Dates")]);
            if self.title_separator {
                report.add_row(text_title_separator!(report.columns()));
            }
            for histories in locations_history_dates {
                if histories.history_dates.is_empty() {
                    // report.text(rptrow!(location_history_dates.location.name));
                    report.add_row(vec![toolslib::text!(histories.location.name), toolslib::text!("None")]);
                } else {
                    let date_format = match &self.date_format {
                        None => "%Y-%m-%d",
                        Some(string) => &string,
                    };
                    macro_rules! to_strings {
                        ($history_range:expr) => {
                            match $history_range.is_one_day() {
                                true => fmt_date(&$history_range.start, date_format),
                                false => format!(
                                    "{} thru {}",
                                    fmt_date(&$history_range.start, date_format),
                                    fmt_date(&$history_range.end, date_format)
                                ),
                            }
                        };
                    }
                    let history_dates = histories.history_dates;
                    report.add_row(vec![
                        toolslib::text!(histories.location.name),
                        toolslib::text!(to_strings!(history_dates[0])),
                    ]);
                    history_dates[1..].into_iter().for_each(|date_range| {
                        report.add_row(vec![toolslib::text!(""), toolslib::text!(to_strings!(date_range))])
                    })
                }
            }
            report
        }
    }
}

pub mod csv {
    //! The list history CSV based reporting implementation.
    //!
    use super::*;
    extern crate csv as csv_lib;

    #[derive(Default, Debug)]
    pub struct Report;
    impl Report {
        /// Generates the list history CSV based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `location_history_dates` - The list of location and history dates that will be reported.
        ///
        pub fn generate(self, locations_history_dates: Vec<HistoryDates>) -> String {
            let mut writer = csv_lib::Writer::from_writer(vec![]);
            csv_write_record!(writer, &["location", "start_date", "end_date"]);
            for location_history_dates in locations_history_dates {
                for history_range in location_history_dates.history_dates {
                    let (from, to) = history_range.as_iso8601();
                    csv_write_record!(writer, &[&location_history_dates.location.name, &from, &to]);
                }
            }
            csv_to_string(writer)
        }
    }
}

pub mod json {
    //! The list history JSON based reporting implementation.
    //!
    use super::*;

    #[derive(Debug, Default)]
    pub struct Report(
        /// Controls if the report will be pretty printed or not.
        bool,
    );
    impl Report {
        /// Create a report instance and configure it to pretty print the `JSON` document.
        ///
        pub fn pretty_printed() -> Self {
            Self(true)
        }
        /// Generates the list history JSON based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `location_history_dates` - The list of location and history dates that will be reported.
        ///
        pub fn generate(&self, locations_history_dates: Vec<HistoryDates>) -> String {
            let location_array: Vec<Value> = locations_history_dates
                .into_iter()
                .map(|location_history_dates| {
                    let history_dates: Vec<Value> = location_history_dates
                        .history_dates
                        .iter()
                        .map(|history_range| {
                            let (from, to) = history_range.as_iso8601();
                            json!({
                                "start": from,
                                "end": to,
                            })
                        })
                        .collect();
                    json!({
                        "location": location_history_dates.location.name,
                        "dates": history_dates,
                    })
                })
                .collect();
            json_to_string(json!({ "history": location_array }), self.0)
        }
    }
}
