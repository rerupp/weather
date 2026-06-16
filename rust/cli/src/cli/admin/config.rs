//! The weather data configuration utility.
//!
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::prelude::Configuration;

/// The configuration command name.
pub const COMMAND_NAME: &'static str = "config";

/// The command argument id used to show the current configuration.
const SHOW: &'static str = "SHOW";

/// The command argument id that will create a default weather data configuration file.
const INIT: &'static str = "INIT";

/// The command argument id that will create a default weather data configuration file.
const FILE: &'static str = "FILE";

/// Get the configuration command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("The weather data configuration utility.")
        .arg(
            Arg::new(INIT)
                .long("init")
                .action(ArgAction::SetTrue)
                .conflicts_with(SHOW)
                .help("Create a default weather data configuration file."),
        )
        .arg(
            Arg::new(FILE)
                .long("file")
                .action(ArgAction::Set)
                .require_equals(true)
                .default_missing_value(Configuration::DEFAULT_FILENAME)
                .default_value(Configuration::DEFAULT_FILENAME)
                .num_args(0..=1)
                .conflicts_with(SHOW)
                .help("The weather data configuration filename."),
        )
        .arg(
            Arg::new(SHOW)
                .long("show")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([INIT, FILE])
                .help("Show the current weather data configuration."),
        )
        .arg_required_else_help(true)
}

/// Collect the command line arguments and run the command.
///
/// # Arguments
///
/// * `configuration` is the current configuration that was loaded.
/// * `args` holds the config command arguments.
///
pub fn execute(configuration: Configuration, args: ArgMatches) -> cli::Result<()> {
    if args.get_flag(SHOW) {
        println!("{configuration}");
    } else if args.get_flag(INIT) {
        let file = std::path::Path::new(args.get_one::<String>(FILE).unwrap());
        configuration.save(file, true)?;
    }
    Ok(())
}
