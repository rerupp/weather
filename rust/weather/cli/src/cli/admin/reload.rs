//! The sync database with archives command.
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::{
    admin_prelude::WeatherAdmin,
    prelude::{location_filter, LocationFilters},
};

#[derive(Debug)]
pub struct ReloadCmd;

impl ReloadCmd {
    /// The sync sub-command name.
    pub const NAME: &'static str = "reload";

    /// The command argument id for which archives should be synced.
    const CRITERIA: &'static str = "CRITERIA";

    /// Get the migrate sub-command definition.
    pub fn get() -> Command {
        Command::new(Self::NAME).about("Reload database weather history for locations.").arg(
            Arg::new(Self::CRITERIA)
                .value_name("LOCATION")
                .action(ArgAction::Append)
                .required(true)
                .help("The locations that will be reloaded (supports wildcards)."),
        )
    }

    /// Collect the command line arguments and run the migrate command.
    ///
    /// # Arguments
    ///
    /// * `admin_api` is the backend weather administration `API`.
    /// * `args` are the reload command arguments.
    ///
    pub fn run(admin_api: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
        let filters = match args.get_many::<String>(Self::CRITERIA) {
            None => LocationFilters::default(),
            Some(locations) => {
                let filters =
                    locations.into_iter().map(|location| location_filter!(name = location)).collect::<Vec<_>>();
                LocationFilters::new(filters)
            }
        };
        let sync_count = admin_api.reload(filters)?;
        log::info!("{} archives converted.", sync_count);
        Ok(())
    }
}
