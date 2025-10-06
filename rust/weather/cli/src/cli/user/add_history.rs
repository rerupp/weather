//! The add weather data history command.

use super::{date_parser, validate_location};
use crate::cli::{self, err};
use chrono::NaiveDate;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::{
    io::{stdout, Write},
    thread,
    time::Duration,
};
use weather_lib::{
    location_filter, location_filters,
    prelude::{DailyHistories, DateRange, HistoriesFuture, WeatherData},
};

/// The add weather data history command name.
pub const COMMAND_NAME: &'static str = "ah";

/// The location argument id.
const LOCATION: &'static str = "LOCATION";

/// The history from date argument id.
const FROM: &'static str = "FROM";

/// The history thru date argument id.
const THRU: &'static str = "THRU";

/// Create a new instance of the add history command arguments.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Add weather history to a location.")
        .arg(
            Arg::new(LOCATION)
                .action(ArgAction::Set)
                .required(true)
                .value_name("LOCATION")
                .value_parser(validate_location)
                .help("The location weather history will be added to."),
        )
        .arg(
            Arg::new(FROM)
                .action(ArgAction::Set)
                .required(true)
                .value_parser(date_parser)
                .value_name("FROM")
                .help("The weather history starting date."),
        )
        .arg(
            Arg::new(THRU)
                .action(ArgAction::Set)
                .required(false)
                .value_parser(date_parser)
                .value_name("THRU")
                .help("The weather history ending date."),
        )
        .arg_required_else_help(true)
}

/// Executes the add history command.
///
/// # Arguments
///
/// * `weather_data` is the weather library API used by the command.
/// * `args` contains the report history command arguments.
///
pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    let location = args.get_one::<String>(LOCATION).unwrap();
    match weather_data.get_locations(location_filters![location_filter!(name = location)]) {
        Err(error) => err!("Error getting location '{location}' information:  {:?}.", error),
        Ok(mut locations) => {
            let len = locations.len();
            if len == 0 {
                err!("Location '{location}' was not found.")
            } else if len > 1 {
                err!("Multiple locations were found for '{location}'.")
            } else {
                let location = locations.pop().unwrap();
                let from = args.get_one::<NaiveDate>(FROM).unwrap();
                let to = args.get_one::<NaiveDate>(THRU).map_or(from, |d| d);
                let date_range = DateRange { start: from.clone(), end: to.clone() };
                let future = weather_data.new_daily_histories(location_filter!(name = &location.alias), date_range)?;
                let daily_histories = get_histories(future)?;
                if daily_histories.is_none() {
                    err!("No daily histories found for {}.", location.name)?;
                }
                let histories_added = weather_data.add_histories(daily_histories.unwrap())?;
                println!("\n{} histories added", histories_added);
                Ok(())
            }
        }
    }
}

fn get_histories(future: HistoriesFuture) -> cli::Result<Option<DailyHistories>> {
    let mut loop_cnt = 0usize;
    let sleep_interval = Duration::from_millis(1);
    loop {
        if future.is_finished() {
            break;
        }
        loop_cnt += 1;
        if (loop_cnt % 100) == 0 {
            write!(stdout().lock(), ".").unwrap();
            stdout().flush().unwrap();
        }
        thread::sleep(sleep_interval);
    }
    match future.get() {
        Ok(daily_histories) => Ok(daily_histories),
        Err(error) => err!("Error getting daily histories:  {error}."),
    }
}
