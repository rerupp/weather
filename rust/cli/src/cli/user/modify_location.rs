//! The update location command
use super::location::{cmd, tui};
use crate::cli;
use clap::{ArgMatches, Command};
use tui_lib::run_viewport;
use weather_lib::prelude::{LocationFilter, WeatherData};

/// The copy sub-command name.
pub const COMMAND_NAME: &'static str = "ml";

/// Get the modify location sub-command definition.
///
pub fn command() -> Command {
    cmd::command(COMMAND_NAME, "Modify the properties of a location.", cmd::CommandMode::Modify)
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
    let alias = cmd::get_alias(&args);
    match weather_data.get_location(LocationFilter::alias(&alias))? {
        None => cli::err!("A location was not found using the alias '{alias}'."),
        Some(current) => {
            macro_rules! run_tui {
                () => {{
                    let mut viewport = tui::LocationEditor::new(tui::LocationEditorMode::Modify(current.clone()));
                    run_viewport(tui::VIEWPORT_ROWS, |terminal| viewport.run(terminal))?
                }};
            }
            let update_opt = match cmd::is_tui(&args) {
                true => run_tui!(),
                false => {
                    // there were no command line arguments present so use the TUI
                    let mut update_opt = cmd::get_location(&args);
                    if update_opt.is_none() {
                        update_opt = run_tui!();
                    }
                    update_opt
                }
            };
            // the RustRover IDE is braindead figuring out that mut update is used
            if let Some(mut update) = update_opt {
                use std::fmt::Write;
                let mut changes = String::new();
                macro_rules! track_change {
                    ($attr: ident) => {
                        // ignore the update attribute if it is empty
                        if update.$attr.len() > 0 {
                            // clear the update attribute if it as not been changed
                            if update.$attr == current.$attr {
                                update.$attr = Default::default();
                            } else {
                                write!(changes, "\n  {}={}", stringify!($attr), update.$attr).unwrap();
                            }
                        }
                    };
                }
                track_change!(country_name);
                track_change!(country_code);
                track_change!(region_name);
                track_change!(region_code);
                track_change!(city_name);
                track_change!(latitude);
                track_change!(longitude);
                track_change!(tz);
                match changes.is_empty() {
                    true => println!("There were no changes made to {current}"),
                    false => {
                        weather_data.update_location(update)?;
                        println!("The following changes were made to {current}:{changes}");
                    }
                }
            }
            Ok(())
        }
    }
}
