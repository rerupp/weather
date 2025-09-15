//! The weather data location reports
use weather_lib::prelude::Location;

use super::{csv_to_string, csv_write_record, json_to_string, text_title_separator};
use serde_json::{json, Value};
use toolslib::{header, layout, report::ReportSheet};

pub mod text {
    /// The list locations_win text based reporting implementation.
    ///
    use super::*;

    /// The metadata controlling the report appearance.
    ///
    #[derive(Default, Debug)]
    pub struct Report {
        /// Controls if a separator row will be added between the report headers and report text.
        title_separator: bool,
        /// Controls if the location alias name will be included in the report or not.
        skip_alias: bool,
    }
    impl Report {
        /// Controls if a separator row will separate report headers from the report text.
        pub fn with_title_separator(mut self) -> Self {
            self.title_separator = true;
            self
        }
        /// Controls if the location alias name will be displayed or not.
        pub fn with_skip_alias(mut self) -> Self {
            self.skip_alias = true;
            self
        }
        /// Generates the list locations_win text based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `locations_win` - The list of locations_win that will be reported.
        ///
        pub fn generate(&self, locations: &Vec<Location>) -> ReportSheet {
            let ll_width = "-###.########".len();
            let mut layouts = vec![];
            layouts.push(layout!(<));
            if !self.skip_alias {
                layouts.push(layout!(<))
            }
            layouts.push(layout!(^ [ll_width * 2 + 1]));
            layouts.push(layout!(<));
            let mut report = ReportSheet::new(layouts);
            let mut headers = vec![];
            headers.push(header!(^ "Location"));
            if !self.skip_alias {
                headers.push(header!(^ "Alias"));
            }
            headers.push(header!(^ " Latitude/Longitude"));
            headers.push(header!(^ "Timezone"));
            report.add_row(headers);
            if self.title_separator {
                report.add_row(text_title_separator!(report.columns()));
            }
            locations.into_iter().for_each(|location| {
                let mut content = vec![];
                content.push(toolslib::text!(location.name.as_str()));
                if !self.skip_alias {
                    content.push(toolslib::text!(location.alias.as_str()))
                }
                content.push(toolslib::text!(format!("{:>ll_width$}/{:<ll_width$}", &location.latitude, &location.longitude)));
                content.push(toolslib::text!(location.tz.as_str()));
                report.add_row(content);
            });
            report
        }
    }
}

pub mod csv {
    /// The list locations_win CSV based reporting implementation.
    ///
    use super::*;
    extern crate csv as csv_lib;

    #[derive(Debug, Default)]
    pub struct Report;
    impl Report {
        /// Generates the list locations_win CSV based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `locations_win` - The list of locations_win that will be reported.
        ///
        pub fn generate(&self, locations: Vec<Location>) -> String {
            let mut writer = csv_lib::Writer::from_writer(vec![]);
            csv_write_record!(writer, &["name", "alias", "longitude", "latitude", "tz"]);
            for location in locations {
                csv_write_record!(
                    writer,
                    &[location.name, location.alias, location.longitude, location.latitude, location.tz,]
                );
            }
            csv_to_string(writer)
        }
    }
}

pub mod json {
    /// The list locations_win JSON based reporting implementation.
    ///
    use super::*;

    #[derive(Default, Debug)]
    pub struct Report(
        /// Controls if the report will be pretty printed or not.
        bool
    );
    impl Report {
        /// Create a report instance and configure it to pretty print the `JSON` document.
        ///
        pub fn pretty_printed() -> Self {
            Self(true)
        }
        /// Generates the list locations_win JSON based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `locations_win` - The list of locations_win that will be reported.
        ///
        pub fn generate(&self, locations: Vec<Location>) -> String {
            let location_array = locations
                .iter()
                .map(|location| {
                    json!({
                        "name": location.name,
                        "alias": location.alias,
                        "longitude": location.longitude,
                        "latitude": location.latitude,
                        "tz": location.tz
                    })
                })
                .collect::<Vec<Value>>();
            let document = json!({ "locations_win": location_array });
            json_to_string(document, self.0)
        }
    }
}
