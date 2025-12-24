//! The delete location command.
//!
use super::get_location;
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::prelude::{LocationFilter, WeatherData};

/// The delete location command name.
pub const COMMAND_NAME: &'static str = "dl";

/// The command argument id for the source archive.
const ALIAS: &'static str = "ALIAS";

/// Get the delete location command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Delete a location from weather history.")
        .arg(Arg::new(ALIAS).action(ArgAction::Set).required(true).value_name(ALIAS).help("The location alias name."))
        .arg_required_else_help(true)
}

/// Collect the command line arguments and run the copy database sub-command.
///
/// # Arguments
///
/// * `weather_admin` is the backend weather administration `API`.
/// * `args` holds the drop command arguments.
///
pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    let alias = args.get_one::<String>(ALIAS).unwrap();
    let on_multiple = || cli::err!("Multiple locations found using '{alias}' as an alias.");
    match get_location(weather_data, alias, on_multiple)? {
        None => cli::err!("A location was not found using '{alias}' as the alias."),
        Some(location) => {
            weather_data.delete_location(LocationFilter::name(&location.alias))?;
            Ok(())
        }
    }
}
