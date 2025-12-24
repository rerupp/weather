//! The drop database utility command.
//! 
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::admin_prelude::WeatherAdmin;

/// The drop database command name.
pub const COMMAND_NAME: &'static str = "drop";

/// The command argument id to remove the existing weather data database file.
const DELETE: &'static str = "DELETE";

/// Get the drop database command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME).about("Delete the existing database schema.").arg(
        Arg::new(DELETE)
            .long("delete")
            .action(ArgAction::SetTrue)
            .help("Remove the database file from the weather data directory."),
    )
}

/// Collect the command line arguments and run the drop database utility command.
///
/// # Arguments
///
/// * `admin_api` is the backend weather administration `API`.
/// * `args` holds the drop command arguments.
///
pub fn execute(admin_api: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
    let delete = args.get_flag(DELETE);
    Ok(admin_api.drop(delete)?)
}
