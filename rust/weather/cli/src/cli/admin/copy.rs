//! The copy location utility command.
//!
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::{admin_prelude::WeatherAdmin, prelude::LocationFilter};

/// The copy location command name.
pub const COMMAND_NAME: &'static str = "copy";

/// The command argument id for the source archive.
const SOURCE: &'static str = "SRC";

/// The command argument id for the source archive.
const DESTINATION: &'static str = "DEST";

/// Get the copy sub-command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Copy a locations weather history to a new location.")
        .arg(
            Arg::new(SOURCE)
                .action(ArgAction::Set)
                .required(true)
                .value_name(SOURCE)
                .help("The source location alias name."),
        )
        .arg(
            Arg::new(DESTINATION)
                .action(ArgAction::Set)
                .required(true)
                .value_name(DESTINATION)
                .help("The destination location alias name."),
        )
        .arg_required_else_help(true)
}

/// Collect the command line arguments and run the copy database sub-command.
///
/// # Arguments
///
/// * `weather_admin` is the backend weather administration `API`.
/// * `args` holds the drop command arguments.
///
pub fn execute(weather_admin: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
    let destination_alias = args.get_one::<String>(DESTINATION).unwrap();
    // make sure the destination alias name is ok
    if destination_alias.contains("*") {
        cli::err!("The destination alias cannot contain wildcards.")?;
    }

    let source_alias = args.get_one::<String>(SOURCE).unwrap();
    match weather_admin.weather_data.get_location(LocationFilter::alias(source_alias))? {
        None => log::warn!("The source location was not found."),
        Some(mut destination) => {
            // the alias is the only difference in the location properties
            destination.alias = destination_alias.to_owned();
            weather_admin.copy_location(source_alias, destination)?;
        }
    }
    Ok(())
}
