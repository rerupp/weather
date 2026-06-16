//! The weather data location reports
use weather_lib::prelude::Location;

use super::{csv_to_string, csv_write_record, json_to_string, text_title_separator};
use serde_json::{json, Value};
use toolslib::{header, layout, report::ReportSheet};

pub mod text {
    /// The list locations text based reporting implementation.
    ///
    use super::*;

    /// The metadata controlling the report appearance.
    ///
    #[derive(Default, Debug)]
    pub struct Report {
        /// When true a separator row will be added between the report headers and report body.
        title_separator: bool,
        /// When true the location alias name will not be included in the report.
        skip_alias: bool,
        /// When true all the location properties will be shown.
        all_properties: bool,
    }
    impl Report {
        /// A builder method that sets the flag to use a separator row between report headers and the report body.
        ///
        pub fn with_title_separator(mut self) -> Self {
            self.title_separator = true;
            self
        }

        /// A builder method that sets the flag to not display the location alias name.
        ///
        pub fn with_skip_alias(mut self) -> Self {
            self.skip_alias = true;
            self
        }

        /// A builder method that sets the flag to show all the location properties.
        ///
        pub fn with_all_properties(mut self) -> Self {
            self.all_properties = true;
            self
        }

        /// Generates the locations text based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `locations` is the collection of location properties that will be used by the report.
        ///
        pub fn generate(&self, locations: &Vec<Location>) -> ReportSheet {
            match self.all_properties {
                true => self.full_report(locations),
                false => self.default_report(locations),
            }
        }

        /// Generates the default locations text based report.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `locations` is the collection of location properties that will be used by the report.
        ///
        fn default_report(&self, locations: &Vec<Location>) -> ReportSheet {
            let ll_width = "-###.########".len();
            let mut layouts = vec![];
            if !self.skip_alias {
                layouts.push(layout!(<))
            }
            layouts.push(layout!(<));
            layouts.push(layout!(^ [ll_width * 2 + 1]));
            layouts.push(layout!(<));

            let mut report = ReportSheet::new(layouts);
            let mut headers = vec![];
            if !self.skip_alias {
                headers.push(header!(^ "Alias"));
            }
            headers.push(header!(^ "City, Region"));
            headers.push(header!(^ " Latitude/Longitude"));
            headers.push(header!(^ "Timezone"));
            report.add_row(headers);
            if self.title_separator {
                report.add_row(text_title_separator!(report.columns()));
            }
            locations.into_iter().for_each(|location| {
                let mut content = vec![];
                if !self.skip_alias {
                    content.push(toolslib::text!(location.alias.as_str()))
                }
                content.push(toolslib::text!(format!("{}, {}", location.city_name, location.region_code)));
                content.push(toolslib::text!(format!(
                    "{:>ll_width$}/{:<ll_width$}",
                    &location.latitude, &location.longitude
                )));
                content.push(toolslib::text!(location.tz.as_str()));
                report.add_row(content);
            });
            report
        }

        /// Generates the locations text based report showing all properties.
        ///
        /// An error will be returned if there are issues writing the report.
        ///
        /// # Arguments
        ///
        /// * `locations` is the collection of location properties that will be used by the report.
        ///
        fn full_report(&self, locations: &Vec<Location>) -> ReportSheet {
            let mut layouts = vec![];
            if !self.skip_alias {
                layouts.push(layout!(<));
            }
            // city name
            layouts.push(layout!(<));
            // region
            layouts.push(layout!(<));
            // latitude/longitude
            let ll_width = "-###.########".len();
            layouts.push(layout!(< [ll_width * 2 + 1]));
            // timezone
            layouts.push(layout!(<));
            // country
            layouts.push(layout!(<));
            let mut report = ReportSheet::new(layouts);
            let mut headers = vec![];
            if !self.skip_alias {
                headers.push(header!(^ "Alias"));
            }
            headers.push(header!(^ "City Name"));
            headers.push(header!(^ "Region"));
            headers.push(header!(< format!("{:>ll_width$}/{:<ll_width$}", "Latitude", "Longitude")));
            headers.push(header!(^ "Timezone"));
            headers.push(header!(^ "Country"));
            report.add_row(headers);
            if self.title_separator {
                report.add_row(text_title_separator!(report.columns()));
            }
            locations.into_iter().for_each(|location| {
                let mut content = vec![];
                if !self.skip_alias {
                    content.push(toolslib::text!(&location.alias))
                }
                content.push(toolslib::text!(format!("{}", location.city_name)));
                content.push(toolslib::text!(&location.region_name));
                content.push(toolslib::text!(format!(
                    "{:>ll_width$}/{:<ll_width$}",
                    &location.latitude, &location.longitude
                )));
                content.push(toolslib::text!(&location.tz));
                content.push(toolslib::text!(format!("{} ({})", location.country_name, location.country_code)));
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
            csv_write_record!(
                writer,
                &[
                    "city_name",
                    "country_name",
                    "country_code",
                    "region_name",
                    "region_code",
                    "alias",
                    "longitude",
                    "latitude",
                    "tz"
                ]
            );
            for location in locations {
                csv_write_record!(
                    writer,
                    &[
                        location.city_name,
                        location.country_name,
                        location.country_code,
                        location.region_name,
                        location.region_code,
                        location.alias,
                        location.longitude,
                        location.latitude,
                        location.tz,
                    ]
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
        bool,
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
                        "city-name": location.city_name,
                        "country-name": location.country_name,
                        "country-code": location.country_code,
                        "region-name": location.region_name,
                        "region-code": location.region_code,
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
