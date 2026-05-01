//! The weather history data consistency check command.
//!
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::fmt::Write;
use weather_lib::admin_prelude::{
    DbHistoryProblems, DbLocationProblems, FilesysDocumentProblem, FilesysLocationProblem, WeatherAdmin,
};

/// The weather history data consistency command name.
pub const COMMAND_NAME: &'static str = "check";

/// The command argument id for the source archive.
const REPAIR: &'static str = "REPAIR";

/// Get the weather history data consistency command.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME).about("Check the weather history data consistency.").arg(
        Arg::new(REPAIR)
            .short('r')
            .long("repair")
            .action(ArgAction::SetTrue)
            .help("Try to fix any inconsistencies found."),
    )
}

/// Run the weather history data consistency command..
///
/// # Arguments
///
/// * `weather_admin` is the backend weather administration `API`.
/// * `args` holds the drop command arguments.
///
pub fn execute(weather_admin: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
    let repair = *args.get_one::<bool>(REPAIR).unwrap();
    let (filesys_problems, db_problems) = weather_admin.check(repair);
    if let Some(problems) = filesys_problems {
        if let Some(problem) = problems.document_problem {
            describe_document_problem(problem);
        } else {
            let has_location_problems = problems.location_problems.is_some();
            if let Some(problems) = problems.location_problems {
                describe_fs_location_problems(problems);
            }
            if let Some(detached_archives) = problems.detached_archives {
                if has_location_problems {
                    eprintln!();
                }
                describe_fs_detached_archives(detached_archives);
            }
        }
    }
    if let Some(db_problems) = db_problems {
        if let Some(db_error) = db_problems.db_error {
            eprintln!("There was an error checking the database: {db_error}.");
        }
        if let Some(location_problems) = db_problems.location_problems {
            describe_db_location_problems(location_problems);
        }
        if let Some(history_problems) = db_problems.history_problems {
            describe_db_history_problems(history_problems);
        }
    }
    Ok(())
}

/// Describe the problem encountered when opening the locations document.
///
/// # Arguments
///
/// * `problem` describes what happened when opening the location document.
///
fn describe_document_problem(problem: FilesysDocumentProblem) {
    if let Some(open_error) = problem.open_error {
        eprintln!("There was a problem opening the locations document: {open_error}");
    } else if let Some(read_error) = problem.read_error {
        eprintln!("There was a problem reading the locations document: {read_error}");
    } else {
        eprintln!("An unknown problem has occurred with the locations document.");
    }
}

/// Describe problems with location weather history archives.
///
/// # Arguments
///
/// * `problems` describes issues with location weather history archives.
///
fn describe_fs_location_problems(problems: Vec<FilesysLocationProblem>) {
    eprintln!("The following location weather history problems were found:");
    for problem in problems {
        let mut description = format!("  {} history archive", problem.location);
        if problem.missing_archive {
            if problem.repaired {
                write!(description, " was missing and has been fixed.").unwrap();
            } else if let Some(create_error) = problem.create_error {
                write!(description, " is missing and could not be created: {create_error}").unwrap();
            } else {
                write!(description, " is missing.").unwrap();
            }
        } else if let Some(open_error) = problem.open_error {
            if problem.repaired {
                write!(description, " was corrupt and has been fixed").unwrap();
            } else if let Some(create_error) = problem.create_error {
                write!(description, " was corrupt and could not be created: {}", create_error).unwrap();
            } else {
                write!(description, " is corrupt: {open_error}.").unwrap();
            }
        } else {
            write!(description, " unknown error: {problem:?}.").unwrap();
        }
        eprintln!("{description}");
    }
}

/// Describe archives that do not have associated locations.
///
/// # Arguments
///
/// * `detached_archives` is the collection of weather data archives that do not have associated locations.
fn describe_fs_detached_archives(detached_archives: Vec<String>) {
    eprintln!("The following archives do not have an associated location:");
    for detached_archive in detached_archives {
        eprintln!("  {detached_archive}");
    }
}

fn describe_db_location_problems(problems: DbLocationProblems) {
    if let Some(missing_locations) = problems.missing_locations {
        eprintln!("The following locations are missing from the database:");
        for location in missing_locations {
            eprintln!("  {location}");
        }
    }
    if let Some(detached_locations) = problems.detached_locations {
        eprintln!("The following locations were not found in the filesystem:");
        for location in detached_locations {
            eprintln!("  {location}");
        }
    }
}

fn describe_db_history_problems(problems: DbHistoryProblems) {
    if let Some(history_problems) = problems.history_problems {
        eprintln!("The following database history counts differ from the filesystem:");
        for problem_details in history_problems {
            if problem_details.db_histories < problem_details.fs_histories {
                let difference = problem_details.fs_histories - problem_details.db_histories;
                eprintln!("  {} is missing {difference} histories.", problem_details.location);
            } else {
                let difference = problem_details.db_histories - problem_details.fs_histories;
                eprintln!("  {} has {difference} more histories.", problem_details.location);
            }
        }
    }
    if let Some(detached_store) = problems.detached_store {
        eprintln!("The following locations do not exist in the filesystem:");
        for history_summaries in detached_store {
            eprintln!("  {}", history_summaries.location);
        }
    }
}
