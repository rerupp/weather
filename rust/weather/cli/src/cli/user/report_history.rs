//! # The implementation for report history (`rh`).
//!
//! The report history command presents historical weather data details.
//! The details shown depend on what command line flags are supplied.
//! The command will show the high and low temperatures for a date by default.
//!
//! Currently only 1 location can be used.
//!
use super::{date_arg, trim_row_end, validate_location};
use crate::cli::{
    self, err, get_writer,
    reports::report_history::{self as reports, ReportSelector},
    ReportArgs,
};
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::prelude::{LocationFilter, WeatherData};

/// The report history command name.
pub const COMMAND_NAME: &'static str = "rh";

/// The report temperature argument id.
///
const TEMPERATURES: &'static str = "TEMPERATURES";

/// The report conditions argument id.
///
const CONDITIONS: &'static str = "CONDITIONS";

/// The report precipitation argument id.
///
const PRECIPITATION: &'static str = "PRECIPITATION";

/// The report summary argument id.
///
const SUMMARY: &'static str = "SUMMARY";

/// The report all argument id.
///
const ALL: &'static str = "ALL";

/// The location argument id.
///
const LOCATION: &'static str = "LOCATION";

/// The history start date argument id.
///
const START: &'static str = "START";

/// The history end date argument id.
///
const END: &'static str = "END";

/// Create the report history command.
///
pub fn command() -> Command {
    let cmd_args = [
        Arg::new(TEMPERATURES)
            .short('t')
            .long("temp")
            .action(ArgAction::SetTrue)
            .conflicts_with(ALL)
            .help("Include temperature information in the report (default)."),
        Arg::new(PRECIPITATION)
            .short('p')
            .long("precip")
            .action(ArgAction::SetTrue)
            .conflicts_with(ALL)
            .help("Include precipitation information in the report."),
        Arg::new(CONDITIONS)
            .short('c')
            .long("cnd")
            .action(ArgAction::SetTrue)
            .conflicts_with(ALL)
            .help("Include weather conditions in the report."),
        Arg::new(SUMMARY)
            .short('s')
            .long("sum")
            .action(ArgAction::SetTrue)
            .conflicts_with(ALL)
            .help("Include summary information in the report."),
        Arg::new(ALL)
            .short('a')
            .long("all")
            .action(ArgAction::SetTrue)
            .help("Include all weather information in the report."),
        Arg::new(LOCATION)
            .action(ArgAction::Set)
            .required(true)
            .value_name("LOCATION")
            .value_parser(validate_location)
            .help("The location to use for the weather history."),
        Arg::new(START)
            .action(ArgAction::Set)
            .required(true)
            .value_name("START")
            .help("The weather history starting date."),
        Arg::new(END).action(ArgAction::Set).required(false).value_name("END").help("The weather history ending date."),
    ];
    Command::new(COMMAND_NAME)
        .about("Generate a weather history report for a location.")
        .long_about(r"Generate a weather history report for a location.

The START and END dates can be specified using any of the following patterns.

  YYYY, MMM-YYYY, MM-YYYY, MM/YYYY, YYYY-MM, YYYY/MM
  MM-DD-YYYY, MM/DD/YYYY, YYYY-MM-DD or YYYY/MM/DD")
        .args(cmd_args)
        .args(ReportArgs::get())
        .group(ReportArgs::arg_group())
        .arg_required_else_help(true)
}

/// Executes the report history command.
///
/// # Arguments
///
/// * `weather_data` is the weather library API used by the command.
/// * `args` contains the report history command arguments.
///
pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    // create the location filter
    let location = args.get_one::<String>(LOCATION).map(|l| l).unwrap();
    let filter = LocationFilter::default().with_name(location);

    // get the report dates
    let start = args.get_one::<String>(START).unwrap();
    let end = args.get_one::<String>(END).map_or(None, |s| Some(s.as_str()));
    let date_range = date_arg::try_parse_daterange(start, end)?;

    // fetch the histories
    let histories = match weather_data.get_daily_histories(filter, date_range) {
        Ok(histories) => histories,
        Err(error) => err!("Error getting daily histories for the report: {error:?}.")?,
    };

    // create the report selection
    let all_content = args.get_flag(ALL);
    let report_selector = ReportSelector {
        temperatures: args.get_flag(TEMPERATURES) || all_content,
        precipitation: args.get_flag(PRECIPITATION) || all_content,
        conditions: args.get_flag(CONDITIONS) || all_content,
        summary: args.get_flag(SUMMARY) || all_content,
    };

    // generate the report
    let report_args = ReportArgs::new(&args);
    let report = if report_args.csv() {
        reports::csv::Report::new(report_selector).generate(histories)
    } else if report_args.json() {
        match report_args.pretty() {
            true => reports::json::Report::pretty_printed(report_selector),
            false => reports::json::Report::new(report_selector),
        }
        .generate(histories)
    } else {
        reports::text::Report::new(report_selector)
            .with_title_separator()
            .generate(histories)
            .into_iter()
            .map(|row| trim_row_end!(row.to_string()))
            .collect::<Vec<String>>()
            .join("\n")
    };
    let mut writer = get_writer(&report_args)?;
    match writer.write_all(report.as_bytes()) {
        Ok(_) => Ok(()),
        Err(error) => err!("Report history error writing report: {:?}", error),
    }
}
