//! The reload a location from the filesystem utility.
//!
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::{admin_prelude::WeatherAdmin, prelude::LocationFilter};

/// The reload a location command name.
pub const COMMAND_NAME: &'static str = "reload";

/// The command option selecting which archives to sync.
const CRITERIA: &'static str = "CRITERIA";

/// Get the reload a location command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME).about("Reload database weather history for locations.").arg(
        Arg::new(CRITERIA)
            .value_name("LOCATION")
            .action(ArgAction::Append)
            .required(true)
            .help("The locations that will be reloaded (supports wildcards)."),
    )
}

/// Collect the command line arguments and run the reload command.
///
/// # Arguments
///
/// * `admin_api` is the backend weather administration `API`.
/// * `args` are the reload command arguments.
///
pub fn execute(admin_api: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
    // at least one location is required by the command
    let filters = args.get_many::<String>(CRITERIA).unwrap().map(|alias| LocationFilter::name(alias)).collect();
    let sync_count = admin_api.reload(filters)?;
    log::info!("{} locations were updated.", sync_count);
    Ok(())
}
