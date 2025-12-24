//! The US Cities administration utility.
//!
use crate::cli::{self, parse_filename};
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::PathBuf;
use toolslib::{
    rptcols, rptrow,
    text::{self, Report},
};
use weather_lib::{
    admin_prelude::{UsCityDetails, WeatherAdmin},
    prelude::Configuration,
};

/// The US cities administration command name.
pub const COMMAND_NAME: &'static str = "uscities";

/// The show information argument id.
const INFO: &'static str = "INFO";

/// The initialize database argument id.
const INIT: &'static str = "INIT";

/// The load database argument id.
const LOAD: &'static str = "LOAD";

/// The delete database argument id.
const DELETE: &'static str = "DELETE";

/// Get the US Cities administration utility command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Administer the US Cities database.")
        .arg(
            Arg::new(INFO)
                .long("info")
                .action(ArgAction::SetTrue)
                .help("Display information about the US Cities database (default)."),
        )
        .arg(Arg::new(INIT).long("init").action(ArgAction::SetTrue).help("Initialize the US Cities database schema."))
        .arg(
            Arg::new(LOAD)
                .long("load")
                .value_name("FILE")
                .value_parser(parse_filename)
                .action(ArgAction::Set)
                .num_args(0..=1)
                .require_equals(true)
                .help("Initialize and load the US Cities file into the database."),
        )
        .arg(Arg::new(DELETE).long("delete").action(ArgAction::SetTrue).help("Delete the US Cities database."))
}

/// Collect the command line arguments and run the migrate command.
///
/// # Arguments
///
/// * `configuration` contains the backend configuration properties.
/// * `args` is the init command arguments.
///
pub fn execute(mut configuration: Configuration, args: ArgMatches) -> cli::Result<()> {
    // update the source filename if one is included
    if let Some(path) = args.get_one::<PathBuf>(LOAD) {
        configuration.us_cities.filename = path.display().to_string();
    }

    // now create the administration API using the updated configuration
    let admin_api = WeatherAdmin::try_from(configuration)?;

    // show details if there are no options
    let mut is_default = true;

    // delete the database?
    if args.get_flag(DELETE) {
        admin_api.uscities_delete()?;
        is_default = false;
    }
    // initialize the database?
    if args.get_flag(INIT) {
        admin_api.uscities_init()?;
        is_default = false;
    }
    // let load the database?
    if args.contains_id(LOAD) {
        admin_api.uscities_load()?;
        is_default = false;
    }
    if args.get_flag(INFO) || is_default {
        if let Some(uscities_info) = admin_api.uscities_info()? {
            report_info(uscities_info)?;
        }
    }
    Ok(())
}

/// Show information about the US Cities database.
///
/// # Arguments
///
/// * `uscities_info` is the detailed information about the database.
///
fn report_info(details: UsCityDetails) -> cli::Result<()> {
    let mut report = Report::from(rptcols!(
        <=(0), ^, >,
        <=(0), ^, >,
        <=(0), ^, >,
        <=(0), ^, >,
        <=(0), ^, >
    ));
    if details.state_info.len() == 0 {
        report.text(rptrow!(="The US Cities database has not been loaded."));
    } else {
        use toolslib::{fmt::commafy, mbufmt, rptdata};
        let mut row = Vec::with_capacity(15);
        for _ in 0..5 {
            row.push(rptdata!(_));
            row.push(rptdata!(^ "State"));
            row.push(rptdata!(^ "Cities"));
        }
        report.header(row);
        report.separator("-");
        let state_cities = details.state_info;
        for base_idx in (0..50).step_by(5) {
            let mut row = Vec::with_capacity(15);
            for (state, cities) in &state_cities[base_idx..base_idx + 5] {
                row.push(rptdata!(_));
                row.push(rptdata!(state.as_str()));
                row.push(rptdata!(commafy(cities)));
            }
            report.text(row);
        }
        let total_cities: usize = state_cities.iter().map(|(_, cities)| cities).sum();
        report.text(rptrow!(=format!("Total cities: {}", mbufmt!(total_cities))));
        report.text(rptrow!(=format!("Database size: {}", mbufmt!(details.db_size))));
    }
    let mut writer = text::get_writer(&None, false)?;
    text::write_strings(&mut writer, report.into_iter())?;
    Ok(())
}
