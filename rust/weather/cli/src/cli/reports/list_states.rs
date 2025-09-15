//! The US City states reports.

use weather_lib::prelude::State;

pub mod text {
    /// The text based states report.
    /// 
    use super::*;
    use crate::cli::reports::text_title_separator;
    use toolslib::{header, layout, report::ReportSheet, text};

    #[derive(Default, Debug)]
    pub struct Report {
        title_separator: bool,
    }
    impl Report {
        /// Controls if a separator row will separate report headers from the report text.
        pub fn with_title_separator() -> Self {
            Self { title_separator: true }
        }

        /// Generates the states JSON based report.
        ///
        /// # Arguments
        ///
        /// * `states` is the collection of US City state metadata.
        ///
        pub fn generate(&self, states: Vec<State>) -> ReportSheet {
            // let ll_width = "-###.########".len();
            let layouts = vec![layout!(<), layout!(^)];
            let mut report = ReportSheet::new(layouts);
            let headers = vec![header!(^ "Name"), header!(^ "State ID")];
            report.add_row(headers);
            if self.title_separator {
                report.add_row(text_title_separator!(report.columns()));
            }
            states.into_iter().for_each(|state| {
                report.add_row(vec![text!(&state.name), text!(&state.state_id)]);
            });
            report
        }
    }
}

pub mod csv {
    /// The CSV based states report.
    /// 
    use super::*;
    use crate::cli::reports::{csv_to_string, csv_write_record};

    extern crate csv as csv_lib;

    #[derive(Default, Debug)]
    pub struct Report;
    impl Report {
        /// Generates the states JSON based report.
        ///
        /// # Arguments
        ///
        /// * `states` is the collection of US City state metadata.
        ///
        pub fn generate(&self, states: Vec<State>) -> String {
            let mut writer = csv_lib::Writer::from_writer(vec![]);
            csv_write_record!(writer, &["name", "state_id"]);
            for state in states {
                csv_write_record!(writer, &[state.name, state.state_id]);
            }
            csv_to_string(writer)
        }
    }
}

pub mod json {
    /// The JSON based states report.
    /// 
    use super::*;
    use crate::cli::reports::json_to_string;
    use serde_json::json;

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

        /// Generates the states JSON based report.
        ///
        /// # Arguments
        ///
        /// * `states` is the collection of US City state metadata.
        ///
        pub fn generate(&self, states: Vec<State>) -> String {
            let json_states = states
                .into_iter()
                .map(|state| {
                    json!({
                        "name": state.name,
                        "state_id": state.state_id,
                    })
                })
                .collect::<Vec<_>>();
            let root = json!({ "states": json_states });
            json_to_string(root, self.0)
        }
    }
}
