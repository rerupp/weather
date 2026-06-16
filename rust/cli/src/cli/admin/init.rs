//! The initialize weather history utility.
//!
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::admin_prelude::WeatherAdmin;

/// The initialize weather history utility command name.
pub const COMMAND_NAME: &'static str = "init";

/// The command argument id indicating initialization should always be run.
const UPDATE: &'static str = "UPDATE";

/// The command argument id indicating the database should be loaded after initialization.
const LOAD: &'static str = "LOAD";

/// The command argument id controlling how many threads to use when loading the database.
const THREADS: &'static str = "THREADS";

/// Get the initialize weather history command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Initialize weather history files and configuration.")
        .arg(
            Arg::new(UPDATE)
                .long("update")
                .action(ArgAction::SetTrue)
                .help("Always initialize the weather data files."),
        )
        .arg(Arg::new(LOAD).long("load").action(ArgAction::SetTrue).help("Load the database after initializing."))
        .arg(
            Arg::new(THREADS)
                .long("threads")
                .action(ArgAction::Set)
                .require_equals(true)
                .value_parser(thread_count_parse)
                .default_value("8")
                .requires(LOAD)
                .help("The number of load threads to use"),
        )
}

/// Collect the command line arguments and run the command.
///
/// # Arguments
///
/// * `admin_api` is the backend weather administration `API`.
/// * `args` holds the initialize command arguments.
///
pub fn execute(admin_api: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
    let load = args.get_flag(LOAD);
    let update = args.get_flag(UPDATE);
    let threads = *args.get_one::<usize>(THREADS).unwrap();
    admin_api.init(update, load, threads)?;
    Ok(())
}

/// Used by the command parser to validate the thread count argument.
///
/// Yeah, I know you can use a builtin but the error message was bugging me.
///
/// # Arguments
///
/// * `dirname` is the weather directory command argument.
///
fn thread_count_parse(count_arg: &str) -> Result<usize, String> {
    match count_arg.parse::<usize>() {
        Ok(count) => {
            let max_threads = 16;
            if count <= max_threads {
                Ok(count)
            } else {
                Err(format!("thread count is limited to {max_threads}."))
            }
        }
        Err(_) => Err(format!("{count_arg} is not a number.")),
    }
}
