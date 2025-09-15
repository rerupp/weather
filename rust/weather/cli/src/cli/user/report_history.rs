//! # The implementation for report history (`rh`).
//!
//! The report history command presents historical weather data details.
//! The details shown depend on what command line flags are supplied.
//! The command will show the high and low temperatures for a date by default.
//!
//! Currently only 1 location can be used.
//!
use super::{date_parser, trim_row_end, validate_location};
use crate::cli::{
    self, err, get_writer,
    reports::report_history::{self as reports, ReportSelector},
    ReportArgs,
};
use chrono::NaiveDate;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::prelude::{DateRange, LocationFilter, WeatherData};

/// The report history command name.
pub(super) const COMMAND_NAME: &'static str = "rh";

pub(super) use v4::{command, execute};
mod v4 {
    //! The current implementation of the report history command.

    use super::*;

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

    /// An internal helper which creates the report selection from the command line arguments.
    ///
    /// # Arguments
    ///
    /// - `args` is the collection of command line arguments.
    ///
    fn create_report_selector(args: &ArgMatches) -> ReportSelector {
        let all_content = args.get_flag(ALL);
        ReportSelector {
            temperatures: args.get_flag(TEMPERATURES) || all_content,
            precipitation: args.get_flag(PRECIPITATION) || all_content,
            conditions: args.get_flag(CONDITIONS) || all_content,
            summary: args.get_flag(SUMMARY) || all_content,
        }
    }

    /// The location argument id.
    ///
    const LOCATION: &'static str = "LOCATION";

    fn get_location(args: &ArgMatches) -> String {
        args.get_one::<String>(LOCATION).map(|location| location.clone()).unwrap()
    }

    /// The history from date argument id.
    ///
    const FROM: &'static str = "FROM";

    fn get_from(args: &ArgMatches) -> NaiveDate {
        args.get_one::<NaiveDate>(FROM).unwrap().clone()
    }

    /// The history thru date argument id.
    ///
    const THRU: &'static str = "THRU";

    fn get_thru(args: &ArgMatches) -> NaiveDate {
        match args.get_one::<NaiveDate>(THRU) {
            None => get_from(args),
            Some(date) => date.clone(),
        }
    }

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
            Arg::new(FROM)
                .action(ArgAction::Set)
                .required(true)
                .value_parser(date_parser)
                .value_name("FROM")
                .help("The weather history starting date."),
            Arg::new(THRU)
                .action(ArgAction::Set)
                .required(false)
                .value_parser(date_parser)
                .value_name("THRU")
                .help("The weather history ending date."),
        ];
        Command::new(COMMAND_NAME)
            .about("Generate a weather history report for a location.")
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
        let filter = LocationFilter::default().with_name(&get_location(&args));
        let date_range = DateRange { start: get_from(&args), end: get_thru(&args) };
        let histories = match weather_data.get_daily_history(filter, date_range) {
            Ok(histories) => histories,
            Err(error) => err!("Report history error getting daily history: {:?}", error)?,
        };
        let report_selector = create_report_selector(&args);
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
}
