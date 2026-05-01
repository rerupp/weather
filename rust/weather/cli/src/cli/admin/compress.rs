//! The compress a locations weather history archive command.
//!
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::{admin_prelude::WeatherAdmin, prelude::LocationFilter};

/// The compress location command name.
pub const COMMAND_NAME: &'static str = "compress";

/// The command argument id for the source archive.
const ALIAS: &'static str = "ALIAS";

/// Get the compress sub-command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Compress a locations weather history archive.")
        .arg(Arg::new(ALIAS).action(ArgAction::Set).required(true).value_name(ALIAS).help("The locations alias name."))
        .arg_required_else_help(true)
}

/// Collect the command line arguments and compress the location archive.
///
/// # Arguments
///
/// * `weather_admin` is the backend weather administration `API`.
/// * `args` holds the compress command arguments.
///
pub fn execute(weather_admin: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
    let alias = args.get_one::<String>(ALIAS).unwrap();
    match weather_admin.weather_data.get_location(LocationFilter::alias(alias))? {
        None => println!("A location was not found using alias '{alias}'."),
        Some(location) => {
            use toolslib::{fmt::commafy, kib};
            let size_difference = weather_admin.compress_archive(&location)?;
            println!("{} bytes were recovered.", commafy(kib!(size_difference, 0)));
        }
    }
    Ok(())
}
