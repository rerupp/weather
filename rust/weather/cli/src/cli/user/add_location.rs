//! The command that will add a location to weather history data.
use super::{get_location, location_args::LocationArgs};
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::prelude::{Location, WeatherData};

/// The add location command name.
pub const COMMAND_NAME: &'static str = "al";

const LOCATION_ALIAS: &'static str = "alias";

/// Create the add location command.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Add a location to weather history.")
        .args(LocationArgs::get(true))
        .arg(
            Arg::new(LOCATION_ALIAS)
                .action(ArgAction::Set)
                .required(true)
                .value_name("ALIAS")
                .help("The location alias."),
        )
        .arg_required_else_help(true)
}

/// Executes the add locations command.
///
/// # Arguments
///
/// * `weather_data` is the weather library API used by the command.
/// * `args` contains the list locations command arguments.
///
pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    let alias = args.get_one::<String>(LOCATION_ALIAS).unwrap().to_string();
    let on_multiple = || cli::err!("Multiple locations found using '{alias}' as an alias.");
    match get_location(weather_data, &alias, on_multiple)? {
        Some(location) => cli::err!("{location} already uses the alias name.")?,
        None => {
            let location_args = LocationArgs::from(&args);
            let new_location = Location {
                city: location_args.city().unwrap(),
                state_id: location_args.state_id().unwrap(),
                state: location_args.state().unwrap(),
                name: Default::default(),
                alias,
                latitude: location_args.latitude().unwrap(),
                longitude: location_args.longitude().unwrap(),
                tz: location_args.tz().unwrap(),
            };
            weather_data.add_location(new_location)?;
            Ok(())
        }
    }
    // match get_location(weather_data, &alias) {
    //     GetLocationResult::Error(error) => cli::err!("{error}"),
    //     GetLocationResult::Multiple => {
    //         println!("Multiple locations were found for '{alias}'.");
    //         Ok(())
    //     }
    //     GetLocationResult::Some(location) => {
    //         println!("{} already uses the alias name.", location);
    //         Ok(())
    //     }
    //     GetLocationResult::None => {
    //         let location_args = LocationArgs::from(&args);
    //         let new_location = Location {
    //             city: location_args.city().unwrap(),
    //             state_id: location_args.state_id().unwrap(),
    //             state: location_args.state().unwrap(),
    //             name: Default::default(),
    //             alias,
    //             latitude: location_args.latitude().unwrap(),
    //             longitude: location_args.longitude().unwrap(),
    //             tz: location_args.tz().unwrap(),
    //         };
    //         weather_data.add_location(new_location)?;
    //         Ok(())
    //     }
    // }
}
