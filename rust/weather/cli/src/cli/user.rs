//! The weather data user CLI commands.
use crate::cli;
use chrono::NaiveDate;
use clap::{ArgMatches, Command};
use weather_lib::prelude::WeatherData;

mod add_history;
mod list_history;
mod list_locations;
mod list_summary;
mod report_history;
mod query_cities;
mod query_states;

#[derive(Debug)]
pub struct User;
impl User {
    /// Return the collection of user commands.
    pub fn get_commands() -> Vec<Command> {
        vec![
            list_locations::command(),
            list_history::command(),
            list_summary::command(),
            report_history::command(),
            add_history::command(),
            query_cities::command(),
            query_states::command(),
        ]
    }
    /// Run the associated command.
    ///
    /// # Arguments
    ///
    /// - `weather_data` is the weather history API that will be used.
    /// - `name` identifies the command that will be run.
    /// - `args` holds the associated command arguments.
    pub fn run(weather_data: &WeatherData, name: &str, args: ArgMatches) -> cli::Result<()> {
        match name {
            list_locations::COMMAND_NAME => list_locations::execute(weather_data, args),
            list_history::COMMAND_NAME => list_history::execute(weather_data, args),
            list_summary::COMMAND_NAME => list_summary::execute(weather_data, args),
            report_history::COMMAND_NAME => report_history::execute(weather_data, args),
            add_history::COMMAND_NAME => add_history::execute(weather_data, args),
            query_cities::COMMAND_NAME => query_cities::execute(weather_data, args),
            query_states::COMMAND_NAME => query_states::execute(weather_data, args),
            _ => unreachable!("User command should not be here..."),
        }
    }
}

/// Validate the location argument to make sure it's not missing.
///
/// # Arguments
///
/// * `name` is the command line argument that should be a location name.
fn validate_location(name: &str) -> Result<String, String> {
    match toolslib::date_time::parse_date(name) {
        Ok(_) => Err("The location name is a date.".to_string()),
        Err(_) => Ok(name.to_string()),
    }
}

/// Parse an argument turning it into a [NaiveDate].
///
/// # Arguments
///
/// * `date` is the argument that will be parsed.
fn date_parser(date: &str) -> Result<NaiveDate, String> {
    match toolslib::date_time::parse_date(date) {
        Ok(date) => Ok(date),
        Err(err) => Err(err.to_string()),
    }
}

/// Trim trailing whitespace from the string.
///
macro_rules! trim_row_end {
    ($string:expr) => {
        $string.trim_end().to_string()
    };
}
use trim_row_end;
