//! The weather data administration cli.
use crate::cli::{self, Command};
use clap::ArgMatches;
use weather_lib::{admin_prelude::WeatherAdmin, prelude::Configuration};

mod check;
mod config;
mod copy;
mod drop;
mod init;
mod reload;
mod show;
mod cities;
mod compress;

#[derive(Debug)]
pub struct Admin;
impl Admin {
    /// The command name.
    pub const NAME: &'static str = "admin";

    /// Create the sub-command.
    ///
    pub fn get() -> Command {
        Command::new(Self::NAME)
            .about("The weather data administration tool.")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .allow_external_subcommands(false)
            .subcommand(init::command())
            .subcommand(check::command())
            .subcommand(drop::command())
            .subcommand(show::command())
            .subcommand(copy::command())
            .subcommand(compress::command())
            .subcommand(reload::command())
            .subcommand(config::command())
            .subcommand(cities::command())
    }

    /// Executes the command.
    ///
    /// # Arguments
    ///
    /// * `configuration` holds the weather data configuration properties.
    /// * `args` has the parsed command arguments.
    ///
    pub fn run(configuration: Configuration, mut args: ArgMatches) -> cli::Result<()> {
        let (name, cmd_args) = args.remove_subcommand().expect("There was no subcommand available to run");
        if name == config::COMMAND_NAME {
            config::execute(configuration, cmd_args)
        } else {
            let weather_admin = WeatherAdmin::try_from(configuration)?;
            match (name.as_str(), cmd_args) {
                (init::COMMAND_NAME, cmd_args) => init::execute(&weather_admin, cmd_args),
                (check::COMMAND_NAME, cmd_args) => check::execute(&weather_admin, cmd_args),
                (drop::COMMAND_NAME, cmd_args) => drop::execute(&weather_admin, cmd_args),
                (show::COMMAND_NAME, cmd_args) => show::execute(&weather_admin, cmd_args),
                (reload::COMMAND_NAME, cmd_args) => reload::execute(&weather_admin, cmd_args),
                (copy::COMMAND_NAME, cmd_args) => copy::execute(&weather_admin, cmd_args),
                (compress::COMMAND_NAME, cmd_args) => compress::execute(&weather_admin, cmd_args),
                (cities::COMMAND_NAME, cmd_args) => cities::execute(&weather_admin, cmd_args),
                _ => unreachable!("Admin command should not be here..."),
            }
        }
    }
}
