//! The drop command
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::admin_prelude::WeatherAdmin;

#[derive(Debug)]
pub struct DropCmd(
    /// The drop command arguments
    ArgMatches,
);

impl DropCmd {
    /// The drop sub-command name.
    pub const NAME: &'static str = "drop";

    /// The command argument id to remove the existing weather data database file.
    const DELETE: &'static str = "DELETE";

    /// Get the drop sub-command definition.
    ///
    pub fn get() -> Command {
        Command::new(Self::NAME).about("Delete the existing database schema.").arg(
            Arg::new(Self::DELETE)
                .long("delete")
                .action(ArgAction::SetTrue)
                .help("Remove the database file from the weather data directory."),
        )
    }

    /// Collect the command line arguments and run the drop database sub-command.
    ///
    /// # Arguments
    ///
    /// * `admin_api` is the backend weather administration `API`.
    /// * `args` holds the drop command arguments.
    ///
    pub fn run(admin_api: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
        let cmd_args = Self(args);
        let delete = cmd_args.delete();
        Ok(admin_api.drop(delete)?)
    }

    /// Get the delete command flag.
    fn delete(&self) -> bool {
        self.0.get_flag(Self::DELETE)
    }
}
