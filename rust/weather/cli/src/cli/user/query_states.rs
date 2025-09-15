//! The user command 'qc' to query US Cities location information.
#![allow(unused)]

use crate::cli::{self, err, get_writer, reports::list_states as reports, user::trim_row_end, ReportArgs};
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::prelude::WeatherData;

/// The query cities command name.
///
pub const COMMAND_NAME: &'static str = "qs";

/// create the query states command.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Get a list of the US City state names.")
        .args(ReportArgs::get())
        .group(ReportArgs::arg_group())
}

pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    match weather_data.get_states() {
        Err(error) => err!("There was an error getting the states: {:?}", error)?,
        Ok(states) => match states.is_empty() {
            true => println!("There were no states found."),
            false => {
                let report_args = ReportArgs::new(&args);
                let mut writer = get_writer(&report_args)?;
                let report = if report_args.csv() {
                    reports::csv::Report::default().generate(states)
                } else if report_args.json() {
                    let report = match report_args.pretty() {
                        true => reports::json::Report::pretty_printed(),
                        false => reports::json::Report::default(),
                    };
                    report.generate(states)
                } else {
                    reports::text::Report::with_title_separator()
                        .generate(states)
                        .into_iter()
                        .map(|row| trim_row_end!(row.to_string()))
                        .collect::<Vec<String>>()
                        .join("\n")
                };
                if let Err(error) = writer.write_all(report.as_bytes()) {
                    err!("List locations error writing the report: {:?}", error)?;
                }
            }
        },
    }
    Ok(())
}