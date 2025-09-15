//! The location history summary report.
//!
use super::{csv_to_string, csv_write_record, json_to_string, text_title_separator};
use serde_json::{json, Value};
use toolslib::{header, layout, report::ReportSheet};
use weather_lib::prelude::HistorySummaries;

pub mod text {
    /// The list summary text based reporting implementation.
    ///
    /// This module utilizes the `text_reports` module to generate reports.
    ///
    use super::*;
    use toolslib::{fmt::commafy, kib};

    /// The metadata controlling the report appearance.
    ///
    #[derive(Debug, Default)]
    pub struct Report(
        /// Controls if a separator row will be added between the report headers and report text.
        bool,
    );
    impl Report {
        /// A builder method that control if a separator row will be added between the report headers and report text.
        pub fn with_title_separator(mut self) -> Self {
            self.0 = true;
            self
        }
        /// Generates the locations_win summary text based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `location_histories` - The list of location history summaries that will be reported.
        ///
        // pub fn generate(location_histories: Vec<HistorySummaries>, writer: &mut impl Write) -> Result<()> {
        pub fn generate(&self, location_histories: Vec<HistorySummaries>) -> ReportSheet {
            let mut report = ReportSheet::new(vec![layout!(<), layout!(>), layout!(>), layout!(>), layout!(>)]);
            report.add_row(vec![
                header!(^ "Location"),
                header!(^ "Overall Size"),
                header!(^ "History Count"),
                header!(^ "History Size"),
                header!(^ "Store Size"),
            ]);
            let columns = report.columns();
            if self.0 {
                report.add_row(text_title_separator!(report.columns()));
            }
            let mut total_size = 0;
            let mut total_history_count = 0;
            let mut total_raw_size = 0;
            let mut total_compressed_size = 0;
            for location_history_summary in location_histories {
                let overall_size = location_history_summary.overall_size.unwrap_or(0);
                let raw_size = location_history_summary.raw_size.unwrap_or(0);
                let compressed_size = location_history_summary.store_size.unwrap_or(0);
                report.add_row(vec![
                    toolslib::text!(location_history_summary.location.name),
                    toolslib::text!(kib!(overall_size, 0)),
                    toolslib::text!(commafy(location_history_summary.count)),
                    toolslib::text!(kib!(raw_size, 0)),
                    toolslib::text!(kib!(compressed_size, 0)),
                ]);
                total_size += overall_size;
                total_history_count += location_history_summary.count;
                total_raw_size += raw_size;
                total_compressed_size += compressed_size;
            }
            report.add_row((0..columns).into_iter().map(|_| toolslib::text!(+ "=")).collect());
            report.add_row(vec![
                header!("Total"),
                toolslib::text!(kib!(total_size, 0)),
                toolslib::text!(commafy(total_history_count)),
                toolslib::text!(kib!(total_raw_size, 0)),
                toolslib::text!(kib!(total_compressed_size, 0)),
            ]);
            report
        }
    }
}

pub mod csv {
    /// The list summary CSV based reporting implementation.
    ///
    use super::*;
    extern crate csv as csv_lib;

    #[derive(Debug, Default)]
    pub struct Report;
    impl Report {
        /// Generates the list summary CSV based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `location_histories` - The list of location history summaries that will be reported.
        ///
        pub fn generate(&self, locations_history_summary: Vec<HistorySummaries>) -> String {
            let mut writer = csv_lib::Writer::from_writer(vec![]);
            csv_write_record!(writer, &["location", "entries", "entries_size", "compressed_size", "size"]);
            for location_history_summary in locations_history_summary {
                let raw_size = location_history_summary.raw_size.map_or(0, |v| v);
                let compressed_size = location_history_summary.store_size.map_or(0, |v| v);
                let overall_size = location_history_summary.overall_size.map_or(0, |v| v);
                csv_write_record!(
                    writer,
                    &[
                        location_history_summary.location.name,
                        location_history_summary.count.to_string(),
                        raw_size.to_string(),
                        compressed_size.to_string(),
                        overall_size.to_string(),
                    ]
                );
            }
            csv_to_string(writer)
        }
    }
}

pub mod json {
    /// The list summary JSON based reporting implementation.
    ///
    use super::*;

    /// The list summary JSON report.
    #[derive(Debug, Default)]
    pub struct Report(
        /// Controls if the `JSON` document will be pretty printed or not.
        bool,
    );
    impl Report {
        /// Create a report instance and configure it to pretty print the `JSON` document.
        ///
        pub fn pretty_printed() -> Self {
            Self(true)
        }
        /// Generates the list summary JSON based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `location_histories` - The list of location history summaries that will be reported.
        ///
        pub fn generate(&self, location_histories: Vec<HistorySummaries>) -> String {
            let location_array: Vec<Value> = location_histories
                .into_iter()
                .map(|location_history_summary| {
                    json!({
                        "location": location_history_summary.location.name,
                        "entries": location_history_summary.count,
                        "entries_size": location_history_summary.raw_size.map_or(0, |v| v),
                        "compressed_size": location_history_summary.store_size.map_or(0, |v| v),
                        "size": location_history_summary.overall_size.map_or(0, |v| v),
                    })
                })
                .collect();
            let root = json!({ "location_summaries": location_array });
            json_to_string(root, self.0)
        }
    }
}
