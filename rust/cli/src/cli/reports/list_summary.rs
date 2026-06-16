//! The location history summary report.
//!
use super::{csv_to_string, csv_write_record, text_title_separator};
use weather_lib::prelude::HistorySummary;

#[derive(Debug, Default)]
pub struct ReportDetails {
    /// Include filesystem details when `true`.
    pub fs_details: bool,
    /// Include database details when `true`.
    pub db_details: bool,
}
impl ReportDetails {
    /// Check if only filesystem details should be shown.
    ///
    #[inline]
    pub fn is_fs_only(&self) -> bool {
        self.fs_details && !self.db_details
    }

    /// Check if only database details should be shown.
    ///
    #[inline]
    pub fn is_db_only(&self) -> bool {
        self.db_details && !self.fs_details
    }

    /// Check if the report only shows details.
    ///
    #[inline]
    pub fn is_details_only(&self) -> bool {
        self.is_fs_only() || self.is_db_only()
    }
}

pub mod text {
    /// The list summary text based reporting implementation.
    ///
    /// This module utilizes the `text_reports` module to generate reports.
    ///
    use super::*;
    use toolslib::{fmt::commafy, header, kib, layout, report::ReportSheet, text};

    /// The metadata controlling the report appearance.
    ///
    #[derive(Debug, Default)]
    pub struct Report {
        /// Controls if a separator row will be added between the report headers and report text.
        separator: bool,
        /// The report details.
        report_details: ReportDetails,
    }
    impl Report {
        /// Create the report with report details.
        ///
        /// # Arguments
        ///
        /// * `report_details` identifies what the contents of the report will contain.
        ///
        pub fn new(report_details: ReportDetails) -> Self {
            Report { report_details, ..Self::default() }
        }

        /// A builder method that control if a separator row will be added between the report headers and report text.
        pub fn with_title_separator(mut self) -> Self {
            self.separator = true;
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
        pub fn generate(&self, history_summaries: Vec<HistorySummary>) -> ReportSheet {
            // remember if the db is included
            let include_db = history_summaries.iter().any(|metadata| metadata.db_history_summary.is_some());

            // create the report layout and headers
            let mut layouts = vec![];
            let mut headers = vec![];
            macro_rules! layout_header {
                ($layout: expr, $header: expr) => {
                    layouts.push($layout);
                    headers.push($header);
                };
            }
            layout_header!(layout!(<), header!(^ "Alias"));
            layout_header!(layout!(<), header!(^ "Location"));
            layout_header!(layout!(>), header!(^ "Days"));
            if !self.report_details.is_db_only() {
                match self.report_details.fs_details {
                    false => {
                        layout_header!(layout!(>), header!(^ "Store Size"));
                    }
                    true => {
                        layout_header!(layout!(>), header!(^ "Uncompressed"));
                        layout_header!(layout!(>), header!(^ "Compressed"));
                        layout_header!(layout!(>), header!(^ "Content"));
                        layout_header!(layout!(>), header!(^ "Archive"));
                    }
                }
            }
            if !self.report_details.is_fs_only() && include_db {
                match self.report_details.db_details {
                    false => {
                        layout_header!(layout!(>), header!(^ "DB Size"));
                    }
                    true => {
                        layout_header!(layout!(>), header!(^ "Data Size"));
                        layout_header!(layout!(>), header!(^ "Data Unused"));
                        layout_header!(layout!(>), header!(^ "Index Size"));
                        layout_header!(layout!(>), header!(^ "Index Unused"));
                    }
                }
            }
            if !self.report_details.is_details_only() {
                layout_header!(layout!(>), header!(^ "Overall Size"));
            }

            // create the report
            let mut report = ReportSheet::new(layouts);
            report.add_row(headers);

            // header separator
            let columns = report.columns();
            if self.separator {
                report.add_row(text_title_separator!(columns));
            }

            // create a helper to make a kib column
            let kib = |v| text!(kib!(std::cmp::max(1, v), 0));

            // the report content
            let mut total_days = 0;
            let mut total_archive_size = 0;
            let mut total_uncompressed_size = 0;
            let mut total_compressed_size = 0;
            let mut total_data_size = 0;
            let mut total_db_size = 0;
            let mut total_data_bytes = 0;
            let mut total_unused_data_bytes = 0;
            let mut total_index_bytes = 0;
            let mut total_unused_index_bytes = 0;
            let mut total_overall_size = 0;
            for history_summary in &history_summaries {
                let mut row = vec![];
                row.push(text!(&history_summary.location.alias));
                let location =
                    format!("{}, {}", history_summary.location.city_name, history_summary.location.region_code);
                row.push(text!(&location));
                row.push(text!(commafy(history_summary.days)));
                total_days += history_summary.days;
                if !self.report_details.is_db_only() {
                    match self.report_details.fs_details {
                        false => {
                            row.push(kib(history_summary.fs_history_summary.archive_size));
                            total_archive_size += history_summary.fs_history_summary.archive_size;
                        }
                        true => {
                            row.push(kib(history_summary.fs_history_summary.uncompressed_size));
                            total_uncompressed_size += history_summary.fs_history_summary.uncompressed_size;
                            row.push(kib(history_summary.fs_history_summary.compressed_size));
                            total_compressed_size += history_summary.fs_history_summary.compressed_size;
                            row.push(kib(history_summary.fs_history_summary.data_size));
                            total_data_size += history_summary.fs_history_summary.data_size;
                            row.push(kib(history_summary.fs_history_summary.archive_size));
                            total_archive_size += history_summary.fs_history_summary.archive_size;
                        }
                    }
                }
                let mut overall_size = history_summary.fs_history_summary.archive_size;

                if !self.report_details.is_fs_only() {
                    if let Some(db_details) = &history_summary.db_history_summary {
                        // the column alignment will be AFU if all locations don't have database metadata
                        let db_size = db_details.data_bytes + db_details.index_bytes;
                        overall_size += db_size;
                        match self.report_details.db_details {
                            false => {
                                row.push(kib(db_size));
                                total_db_size += db_size;
                            }
                            true => {
                                row.push(kib(db_details.data_bytes));
                                total_data_bytes += db_details.data_bytes;
                                row.push(kib(db_details.unused_data_bytes));
                                total_unused_data_bytes += db_details.unused_data_bytes;
                                row.push(kib(db_details.index_bytes));
                                total_index_bytes += db_details.index_bytes;
                                row.push(kib(db_details.unused_index_bytes));
                                total_unused_index_bytes += db_details.unused_index_bytes;
                            }
                        }
                    }
                }
                if !self.report_details.is_details_only() {
                    row.push(text!(kib!(overall_size, 0)));
                }
                total_overall_size += overall_size;
                report.add_row(row);
            }

            // totals
            report.add_row((0..columns).into_iter().map(|_| toolslib::text!(+ "=")).collect());
            let mut totals = vec![];
            totals.push(text!("Total"));
            totals.push(text!(""));
            totals.push(text!(commafy(total_days)));
            if !self.report_details.is_db_only() {
                match self.report_details.fs_details {
                    false => {
                        totals.push(kib(total_archive_size));
                    }
                    true => {
                        totals.push(kib(total_uncompressed_size));
                        totals.push(kib(total_compressed_size));
                        totals.push(kib(total_data_size));
                        totals.push(kib(total_archive_size));
                    }
                }
            }
            if !self.report_details.is_fs_only() && include_db {
                match self.report_details.db_details {
                    false => {
                        totals.push(kib(total_db_size));
                    }
                    true => {
                        totals.push(kib(total_data_bytes));
                        totals.push(kib(total_unused_data_bytes));
                        totals.push(kib(total_index_bytes));
                        totals.push(kib(total_unused_index_bytes));
                    }
                }
            }
            if !self.report_details.is_details_only() {
                totals.push(kib(total_overall_size));
            }
            report.add_row(totals);

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
    pub struct Report {
        /// The report details that manage the report content.
        report_details: ReportDetails,
    }
    impl Report {
        /// Create a new instance of the CSV report generator.
        ///
        /// # Arguments
        ///
        /// * `report_details` identifies the contents of the report.
        ///
        pub fn new(report_details: ReportDetails) -> Self {
            Self { report_details }
        }

        /// Generates the list summary CSV based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `location_histories` - The list of location history summaries that will be reported.
        ///
        pub fn generate(&self, history_summaries: Vec<HistorySummary>) -> String {
            // remember if the db is included
            let include_db = history_summaries.iter().any(|hs| hs.db_history_summary.is_some());

            let mut writer = csv_lib::Writer::from_writer(vec![]);
            let mut headers = vec!["alias", "city", "region", "country", "days"];
            if !self.report_details.is_db_only() {
                match self.report_details.fs_details {
                    false => {
                        headers.push("store_size");
                    }
                    true => {
                        headers.push("uncompressed");
                        headers.push("compressed");
                        headers.push("content");
                        headers.push("archive");
                    }
                }
            }
            if !self.report_details.is_fs_only() && include_db {
                match self.report_details.db_details {
                    false => {
                        headers.push("db_size");
                    }
                    true => {
                        headers.push("data_size");
                        headers.push("data_unused");
                        headers.push("index_size");
                        headers.push("index_unused");
                    }
                }
            }
            if !self.report_details.is_details_only() {
                headers.push("overall_size");
            }
            csv_write_record!(writer, &headers);
            let mut record = vec![];
            for history_summary in history_summaries {
                record.push(history_summary.location.alias.to_string());
                record.push(history_summary.location.city_name.to_string());
                record.push(history_summary.location.region_code.to_string());
                record.push(history_summary.location.country_code.to_string());
                record.push(history_summary.days.to_string());
                let mut overall_size = 0;
                if !self.report_details.is_db_only() {
                    overall_size += history_summary.fs_history_summary.archive_size;
                    match self.report_details.fs_details {
                        false => {
                            record.push(history_summary.fs_history_summary.archive_size.to_string());
                        }
                        true => {
                            record.push(history_summary.fs_history_summary.uncompressed_size.to_string());
                            record.push(history_summary.fs_history_summary.compressed_size.to_string());
                            record.push(history_summary.fs_history_summary.data_size.to_string());
                            record.push(history_summary.fs_history_summary.archive_size.to_string());
                        }
                    }
                }
                if !self.report_details.is_fs_only() && include_db {
                    if let Some(db_summary) = history_summary.db_history_summary {
                        // there will be column data missing if all locations don't have database metadata
                        overall_size += db_summary.data_bytes + db_summary.index_bytes;
                        match self.report_details.db_details {
                            false => {
                                record.push((db_summary.data_bytes + db_summary.index_bytes).to_string());
                            }
                            true => {
                                record.push(db_summary.data_bytes.to_string());
                                record.push(db_summary.unused_data_bytes.to_string());
                                record.push(db_summary.index_bytes.to_string());
                                record.push(db_summary.unused_index_bytes.to_string());
                            }
                        }
                    }
                }
                if !self.report_details.is_details_only() {
                    record.push(overall_size.to_string());
                }
                csv_write_record!(writer, &record);
                record.clear();
            }
            csv_to_string(writer)
        }
    }
}

pub mod json {
    /// The list summary JSON based reporting implementation.
    ///
    use super::*;
    use crate::cli::reports::json_to_string;
    use serde_json::{Map, Value};

    /// The list summary JSON report.
    #[derive(Debug, Default)]
    pub struct Report {
        /// Controls if the `JSON` document will be pretty printed or not.
        pretty_print: bool,
        /// The report details that manage the report content.
        report_details: ReportDetails,
    }
    impl Report {
        /// Create a new instance of the JSON report generator.
        ///
        /// # Arguments
        ///
        /// * `report_details` identifies the contents of the report.
        ///
        pub fn new(report_details: ReportDetails) -> Self {
            Self { report_details, ..Self::default() }
        }

        /// Create a report instance and configure it to pretty print the `JSON` document.
        ///
        pub fn with_pretty_print(mut self, pretty_print: bool) -> Self {
            self.pretty_print = pretty_print;
            self
        }

        /// Generates the list summary JSON based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `history_summaries` - The list of location history summaries that will be reported.
        ///
        pub fn generate(&self, history_summaries: Vec<HistorySummary>) -> String {
            // remember if the db is included
            let include_db = history_summaries.iter().any(|hs| hs.db_history_summary.is_some());

            let mut json_summaries: Vec<Value> = vec![];
            for history_summary in history_summaries {
                let mut json_summary = Map::new();
                macro_rules! add_string {
                    ($key: literal, $string: expr) => {
                        json_summary.insert($key.into(), Value::String($string.to_string()))
                    };
                }
                macro_rules! add_number {
                    ($key: literal, $number: expr) => {
                        json_summary.insert($key.into(), Value::Number($number.into()))
                    };
                }
                add_string!("alias", history_summary.location.alias);
                add_string!("city", history_summary.location.city_name);
                add_string!("region", history_summary.location.region_code);
                add_string!("country", history_summary.location.country_code);
                add_number!("days", history_summary.days);
                let mut overall_size = 0;
                if !self.report_details.is_db_only() {
                    overall_size += history_summary.fs_history_summary.archive_size;
                    match self.report_details.fs_details {
                        false => {
                            add_number!("store_size", history_summary.fs_history_summary.archive_size);
                        }
                        true => {
                            add_number!("uncompress", history_summary.fs_history_summary.uncompressed_size);
                            add_number!("compressed", history_summary.fs_history_summary.compressed_size);
                            add_number!("content", history_summary.fs_history_summary.data_size);
                            add_number!("archive", history_summary.fs_history_summary.archive_size);
                        }
                    }
                }
                if !self.report_details.is_fs_only() && include_db {
                    if let Some(db_summary) = history_summary.db_history_summary {
                        overall_size += db_summary.data_bytes + db_summary.index_bytes;
                        match self.report_details.db_details {
                            false => {
                                add_number!("db_size", db_summary.data_bytes + db_summary.index_bytes);
                            }
                            true => {
                                add_number!("data_size", db_summary.data_bytes);
                                add_number!("data_unused", db_summary.unused_data_bytes);
                                add_number!("index_size", db_summary.index_bytes);
                                add_number!("index_unused", db_summary.unused_index_bytes);
                            }
                        }
                    }
                }
                if !self.report_details.is_details_only() {
                    add_number!("overall_size", overall_size);
                }
                json_summaries.push(Value::Object(json_summary));
            }
            // let json_contents = Value::Array(json_summaries);
            let mut json_document = Map::new();
            json_document.insert("history_summaries".to_string(), Value::Array(json_summaries));
            json_to_string(Value::Object(json_document), true)
        }
    }
}
