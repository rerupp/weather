//! The command that will add a location to weather history data.
use super::location::{cmd, tui};
use crate::cli::{self, user::location::cmd::is_tui};
use clap::{ArgMatches, Command};
use tui_lib::run_viewport;
use weather_lib::prelude::{LocationFilter, WeatherData};

/// The add location command name.
pub const COMMAND_NAME: &'static str = "al";

/// Create the add location command.
///
pub fn command() -> Command {
    cmd::command(COMMAND_NAME, "Add a location to weather history", cmd::CommandMode::Add)
}

/// Executes the add locations command.
///
/// # Arguments
///
/// * `weather_data` is the weather library API used by the command.
/// * `args` contains the list locations command arguments.
///
pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    // check to see if the alias is being used
    let alias = cmd::get_alias(&args);
    match weather_data.get_location(LocationFilter::alias(&alias))? {
        Some(location) => cli::err!("{location} is already using the alias name."),
        None => {
            macro_rules! run_tui {
                () => {{
                    let mut viewport = tui::LocationEditor::new(tui::LocationEditorMode::New(alias));
                    run_viewport(tui::VIEWPORT_ROWS, |terminal| viewport.run(terminal))?
                }};
            }
            let location_opt = match is_tui(&args) {
                true => run_tui!(),
                false => {
                    let mut location_opt = cmd::get_location(&args);
                    // there were no command line arguments present so use the TUI
                    if location_opt.is_none() {
                        location_opt = run_tui!();
                    }
                    location_opt
                }
            };
            if let Some(location) = location_opt {
                weather_data.add_location(location.clone())?;
                println!("{location} added.");
            }
            Ok(())
        }
    }
}
