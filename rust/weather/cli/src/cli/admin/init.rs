//! The weather data initialization command.
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::admin_prelude::WeatherAdmin;

#[derive(Debug)]
pub struct InitCmd(
    /// The init command arguments.
    ArgMatches,
);
impl InitCmd {
    /// The initialize sub-command name.
    pub const NAME: &'static str = "init";

    /// The command argument id indicating the database schema should be dropped.
    const DROP: &'static str = "DROP";

    /// The command argument id indicating the database should be loaded.
    const LOAD: &'static str = "LOAD";

    /// The command argument id controlling how many threads to use.
    const THREADS: &'static str = "THREADS";

    /// Get the initialize sub-command definition.
    ///
    pub fn get() -> Command {
        Command::new(Self::NAME)
            .about("Initialize the weather data database.")
            .arg(
                Arg::new(Self::THREADS)
                    .long("threads")
                    .action(ArgAction::Set)
                    .value_parser(Self::thread_count_parse)
                    .default_value("8")
                    .help("The number of threads to use"),
            )
            .arg(
                Arg::new(Self::DROP)
                    .long("drop")
                    .action(ArgAction::SetTrue)
                    .help("Drops the database before initializing."),
            )
            .arg(
                Arg::new(Self::LOAD)
                    .long("load")
                    .action(ArgAction::SetTrue)
                    .help("Load the database after initializing."),
            )
    }

    /// Collect the command line arguments and run the command.
    ///
    /// # Arguments
    ///
    /// * `admin_api` is the backend weather administration `API`.
    /// * `args` holds the initialize command arguments.
    ///
    pub fn run(admin_api: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
        let cmd_args = Self(args);
        // this is safe, the thread parse already confirms it's a usize
        let threads = cmd_args.threads();
        let drop = cmd_args.drop();
        let load = cmd_args.load();
        admin_api.init(drop, load, threads)?;
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

    /// Get the threads command flag.
    fn threads(&self) -> usize {
        *self.0.get_one(Self::THREADS).unwrap()
    }

    /// Get the drop command flag.
    fn drop(&self) -> bool {
        self.0.get_flag(Self::DROP)
    }

    /// Get the load command flag.
    fn load(&self) -> bool {
        self.0.get_flag(Self::LOAD)
    }
}
