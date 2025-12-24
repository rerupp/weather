//! The update location command
use super::{get_location, location_args::LocationArgs};
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::fmt::Write;
use weather_lib::prelude::{Location, WeatherData};

/// Create the boilerplate code that generates an error.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(cli::Error(format!($($arg)*)))
    };
}

/// The copy sub-command name.
pub const COMMAND_NAME: &'static str = "ml";

/// The command argument id for the locations name.
const ALIAS: &'static str = "ALIAS";

/// Get the modify location sub-command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Modify the properties of a location.")
        .args(LocationArgs::get(false))
        .arg(
            Arg::new(ALIAS)
                .action(ArgAction::Set)
                .required(true)
                .value_name("NAME")
                .help("The location that will be updated."),
        )
        .arg_required_else_help(true)
}

/// Collect the command line arguments and run the copy database sub-command.
///
/// # Arguments
///
/// * `weather_data` is the weather history data `API`.
/// * `args` contains the update command arguments.
///
pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    // make sure there is a location
    let alias = args.get_one::<String>(ALIAS).unwrap();
    let on_multiple = || err!("Multiple locations found using '{alias}' as an alias name.");
    match get_location(weather_data, alias, on_multiple)? {
        None => err!("A location was not found using '{alias}' as an alias name."),
        Some(location) => {
            let location_args = LocationArgs::from(&args);
            if location_args.is_none() {
                err!("At least one command option is required.")
            } else {
                modify_location(location, location_args, weather_data)
            }
        }
    }
}

fn modify_location(location: Location, args: LocationArgs, weather_data: &WeatherData) -> cli::Result<()> {
    let location_update = Location {
        city: args.city().unwrap_or(Default::default()),
        state_id: args.state_id().unwrap_or(Default::default()),
        state: args.state().unwrap_or(Default::default()),
        name: Default::default(),
        alias: location.alias.to_string(),
        latitude: args.latitude().unwrap_or(Default::default()),
        longitude: args.longitude().unwrap_or(Default::default()),
        tz: args.tz().unwrap_or(Default::default()),
    };
    match weather_data.update_location(location_update)? {
        false => {
            println!("There were no updates made to {location}.");
            Ok(())
        }
        true => {
            let on_multiple = || err!("Yikes! Multiple locations for '{}' found after update.", location.alias);
            match get_location(weather_data, &location.alias, on_multiple)? {
                // GetLocationResult::Error(error) => err!("{error}"),
                // GetLocationResult::None => err!("Yikes! {location} was not found after update."),
                // GetLocationResult::Multiple => {
                //     err!("Yikes! Multiple locations for '{}' found after update.", location.alias)
                // }
                // GetLocationResult::Some(update) => {
                //     let mut updates = String::default();
                //     macro_rules! add_update {
                //     ($what: expr, $attr: ident) => {
                //         if location.$attr != update.$attr {
                //             write!(updates, "\n  {}='{}'", $what, update.$attr).unwrap();
                //         }
                //     };
                // }
                //     add_update!("city", city);
                //     add_update!("state_id", state_id);
                //     add_update!("state", state);
                //     add_update!("latitude", latitude);
                //     add_update!("longitude", longitude);
                //     add_update!("tz", tz);
                //     if updates.len() > 0 {
                //         println!("The following updates were made:{updates}");
                //     }
                //     Ok(())
                // }
                None => err!("Yikes! {location} was not found after update."),
                // RustRover is braindead when it comes to understanding update is used in a macro
                #[allow(unused)]
                Some(update) => {
                    let mut updates = String::default();
                    macro_rules! add_update {
                        ($what: expr, $attr: ident) => {
                            if location.$attr != update.$attr {
                                write!(updates, "\n  {}='{}'", $what, update.$attr).unwrap();
                            }
                        };
                    }
                    add_update!("city", city);
                    add_update!("state_id", state_id);
                    add_update!("state", state);
                    add_update!("latitude", latitude);
                    add_update!("longitude", longitude);
                    add_update!("tz", tz);
                    if updates.len() > 0 {
                        println!("The following updates were made:{updates}");
                    }
                    Ok(())
                }
            }
        }
        // true => match get_location(weather_data, &location.alias) {
        //     GetLocationResult::Error(error) => err!("{error}"),
        //     GetLocationResult::None => err!("Yikes! {location} was not found after update."),
        //     GetLocationResult::Multiple => {
        //         err!("Yikes! Multiple locations for '{}' found after update.", location.alias)
        //     }
        //     GetLocationResult::Some(update) => {
        //         let mut updates = String::default();
        //         macro_rules! add_update {
        //             ($what: expr, $attr: ident) => {
        //                 if location.$attr != update.$attr {
        //                     write!(updates, "\n  {}='{}'", $what, update.$attr).unwrap();
        //                 }
        //             };
        //         }
        //         add_update!("city", city);
        //         add_update!("state_id", state_id);
        //         add_update!("state", state);
        //         add_update!("latitude", latitude);
        //         add_update!("longitude", longitude);
        //         add_update!("tz", tz);
        //         if updates.len() > 0 {
        //             println!("The following updates were made:{updates}");
        //         }
        //         Ok(())
        //     }
        // },
    }
}
