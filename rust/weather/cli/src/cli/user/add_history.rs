//! The add weather data history command.

use super::{date_arg, validate_location};
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::{
    io::{stdout, Write},
    thread,
    time::Duration,
};
use weather_lib::prelude::{DailyHistories, HistoriesFuture, LocationFilter, WeatherData};

/// The add weather data history command name.
pub const COMMAND_NAME: &'static str = "ah";

/// The location argument id.
const ALIAS: &'static str = "ALIAS";

/// The history from date argument id.
const START: &'static str = "START";

/// The history thru date argument id.
const END: &'static str = "END";

/// Create a new instance of the add history command arguments.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Add new weather history to a location.")
        .long_about(
            r"Add new weather history to a location.

The START and END dates can be specified using any of the following patterns.

  YYYY, MMM-YYYY, MM-YYYY, MM/YYYY, YYYY-MM, YYYY/MM
  MM-DD-YYYY, MM/DD/YYYY, YYYY-MM-DD, YYYY/MM/DD
  MMM-DD-YYYY, MMM/DD/YYYY",
        )
        .arg(
            Arg::new(ALIAS)
                .action(ArgAction::Set)
                .required(true)
                .value_name("ALIAS")
                .value_parser(validate_location)
                .help("The location weather history will be added to."),
        )
        .arg(
            Arg::new(START)
                .action(ArgAction::Set)
                .required(true)
                .value_name("START")
                .help("The weather history starting date."),
        )
        .arg(
            Arg::new(END)
                .action(ArgAction::Set)
                .required(false)
                .value_name("END")
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
    let location = args.get_one::<String>(ALIAS).unwrap();
    match weather_data.get_locations(Some(vec![LocationFilter::alias(location)])) {
        Err(error) => cli::err!("Error getting location '{location}' properties: {error:?}."),
        Ok(mut locations) => {
            let location = match locations.len() {
                1 => locations.remove(0),
                0 => cli::err!("Location '{location}' was not found.")?,
                _ => cli::err!("Multiple locations were found for '{location}'.")?,
            };
            let start_arg = args.get_one::<String>(START).unwrap();
            let end_arg = args.get_one::<String>(END).map_or(None, |end| Some(end.as_str()));
            let date_range = date_arg::try_parse_daterange(start_arg, end_arg)?;
            let future = weather_data.fetch_daily_histories(LocationFilter::alias(&location.alias), date_range)?;
            let daily_histories = get_histories(future)?;
            if daily_histories.is_none() {
                cli::err!("No daily histories found for {location}.")?;
            }
            let histories_added = weather_data.add_histories(daily_histories.unwrap())?;
            println!("\n{} histories added", histories_added);
            Ok(())
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
        Err(error) => cli::err!("Error getting daily histories:  {error}."),
    }
}
