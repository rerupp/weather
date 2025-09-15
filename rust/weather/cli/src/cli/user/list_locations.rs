//! The list location command implementation.
//!
use super::trim_row_end;
use crate::cli::{
    self, err, get_writer, reports::list_locations as reports, LocationFilterArgs, ReportArgs,
};
use clap::{ArgMatches, Command};
use weather_lib::prelude::WeatherData;

/// The list locations command name.
pub const COMMAND_NAME: &'static str = "ll";

/// Create the list locations command.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("List the known weather data history locations_win.")
        .args(ReportArgs::get())
        .group(ReportArgs::arg_group())
        .args(LocationFilterArgs::get())
}

/// Executes the list locations command.
///
/// # Arguments
///
/// * `weather_data` is the weather library API used by the command.
/// * `args` contains the list locations command arguments.
///
pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    let filters = LocationFilterArgs::new(&args).as_location_filters();
    let locations = weather_data.get_locations(filters)?;
    match locations.is_empty() {
        true => Ok(()),
        false => {
            let report_args = ReportArgs::new(&args);
            let mut writer = get_writer(&report_args)?;
            let report = if report_args.csv() {
                reports::csv::Report::default().generate(locations)
            } else if report_args.json() {
                let report = match report_args.pretty() {
                    true => reports::json::Report::pretty_printed(),
                    false => reports::json::Report::default(),
                };
                report.generate(locations)
            } else {
                reports::text::Report::default()
                    .with_title_separator()
                    .generate(&locations)
                    .into_iter()
                    .map(|row| trim_row_end!(row.to_string()))
                    .collect::<Vec<String>>()
                    .join("\n")
            };
            match writer.write_all(report.as_bytes()) {
                Ok(_) => Ok(()),
                Err(err) => err!("List locations error writing the report: {:?}", err),
            }
        }
    }
}
