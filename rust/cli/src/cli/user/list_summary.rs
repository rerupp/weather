//! # The implementation for list summary (`ls`).
//!
//! The list summary command presents the amount of weather data available. The information
//! includes:
//!
//! * location name
//! * the count of how many historical weather data entries there are
//! * the overall size of weather data
//! * the total size of raw data
//! * the size of the data when compressed
//!
//! The command allows locations to be filtered. The filtering is case-insensitive
//! and will match location alias, city name, region, and country names.
//!
use super::location_filters;
use crate::cli::{
    user::trim_row_end,
    self, err, get_writer, reports::list_summary as reports, ReportArgs,
};
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::prelude::WeatherData;

/// The list summary command name.
///
pub const COMMAND_NAME: &'static str = "ls";

/// The all details argument.
const ALL: &'static str = "all";

/// The database details argument.
const DB_DETAILS: &'static str ="DB";

/// The filesystem details argument.
const FS_DETAILS: &'static str ="FS";

/// create the list summary command.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("List a summary of weather history by location.")
        .arg(Arg::new(FS_DETAILS)
            .long("fs")
            .action(ArgAction::SetTrue)
            .help("Include filesystem details.")
        )
        .arg(Arg::new(DB_DETAILS)
            .long("db")
            .action(ArgAction::SetTrue)
            .help("Include database details.")
        )
        .arg(Arg::new(ALL)
            .short('a')
            .long("all")
            .action(ArgAction::SetTrue)
            .help("Show all details.")
        )
        .args(ReportArgs::get())
        .group(ReportArgs::arg_group())
        .arg(location_filters::arg())
}

/// Executes the list summary command.
///
/// # Arguments
///
/// * `weather_data` is the weather library API used by the command.
/// * `args` contains the list summary command arguments.
///
pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    let filters_opt = location_filters::parse_args(&args)?;
    let history_summaries = weather_data.get_history_summaries(filters_opt)?;
    let all = args.get_flag(ALL);
    let report_details = reports::ReportDetails {
        fs_details: all || args.get_flag(FS_DETAILS),
        db_details: all || args.get_flag(DB_DETAILS),
    };
    match history_summaries.is_empty() {
        true => Ok(()),
        false => {
            let report_args = ReportArgs::new(&args);
            let report = if report_args.csv() {
                reports::csv::Report::new(report_details).generate(history_summaries)
            } else if report_args.json() {
                reports::json::Report::new(report_details)
                    .with_pretty_print(report_args.pretty())
                    .generate(history_summaries)
            } else {
                reports::text::Report::new(report_details)
                    .with_title_separator()
                    .generate(history_summaries)
                    .into_iter()
                    .map(|row| trim_row_end!(row.to_string()))
                    .collect::<Vec<String>>()
                    .join("\n")
            };
            let mut writer = get_writer(&report_args)?;
            match writer.write_all(report.as_bytes()) {
                Ok(_) => Ok(()),
                Err(error) => err!("List summary error writing the report: {:?}", error),
            }
        }
    }
}
