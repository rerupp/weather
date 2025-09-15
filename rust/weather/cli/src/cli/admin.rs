//! The weather data administration cli.
use crate::cli::{self, Command};
use clap::ArgMatches;
use weather_lib::admin_prelude::WeatherAdmin;

mod drop;
use drop::DropCmd;

mod init;
use init::InitCmd;

mod reload;
use reload::ReloadCmd;

mod show;
use show::ShowCmd;

mod us_cities;
use us_cities::UsCitiesCmd;

#[derive(Debug)]
pub struct Admin;
impl Admin {
    /// The command name.
    pub const NAME: &'static str = "admin";
    /// Create the sub-command.
    pub fn get() -> Command {
        Command::new(Self::NAME)
            .about("The weather data administration tool.")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .allow_external_subcommands(false)
            .subcommand(InitCmd::get())
            .subcommand(DropCmd::get())
            .subcommand(ReloadCmd::get())
            .subcommand(ShowCmd::get())
            .subcommand(UsCitiesCmd::get())
    }
    /// Executes the command.
    ///
    /// # Arguments
    ///
    /// * `weather_data` is the weather data API.
    /// * `args` contains the report history command arguments.
    pub fn run(weather_admin: &WeatherAdmin, mut args: ArgMatches) -> cli::Result<()> {
        let (name, cmd_args) = args.remove_subcommand().expect("There was no subcommand available to run");
        match (name.as_str(), cmd_args) {
            (InitCmd::NAME, cmd_args) => InitCmd::run(weather_admin, cmd_args),
            (DropCmd::NAME, cmd_args) => DropCmd::run(weather_admin, cmd_args),
            (ShowCmd::NAME, cmd_args) => ShowCmd::run(weather_admin, cmd_args),
            (ReloadCmd::NAME, cmd_args) => ReloadCmd::run(weather_admin, cmd_args),
            (UsCitiesCmd::NAME, cmd_args) => UsCitiesCmd::run(weather_admin, cmd_args),
            _ => unreachable!("Admin command should not be here..."),
        }
    }
}
