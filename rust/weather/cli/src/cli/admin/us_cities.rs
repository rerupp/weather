//! The US Cities administration command.
use crate::cli::{self, parse_filename};
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::PathBuf;
use toolslib::{
    rptcols, rptrow,
    text::{self, Report},
};
use weather_lib::admin_prelude::{UsCityDetails, WeatherAdmin};

#[derive(Debug)]
pub struct UsCitiesCmd(
    /// The show command arguments
    ArgMatches,
);

impl UsCitiesCmd {
    /// The command name.
    pub const NAME: &'static str = "uscities";

    /// The information argument id.
    const INFO: &'static str = "INFO";

    /// The load argument id.
    const LOAD: &'static str = "LOAD";

    /// The delete argument id.
    const DELETE: &'static str = "DELETE";

    /// Get the US Cities sub-command.
    ///
    pub fn get() -> Command {
        Command::new(Self::NAME)
            .about("Administer the US Cities database.")
            .arg(
                Arg::new(Self::INFO)
                    .long("info")
                    .action(ArgAction::SetTrue)
                    .help("Display information about the US Cities database (default)."),
            )
            .arg(
                Arg::new(Self::LOAD)
                    .long("load")
                    .value_name("FILE")
                    .value_parser(parse_filename)
                    .action(ArgAction::Set)
                    .require_equals(true)
                    .help("Initialize and load the US Cities file into the database."),
            )
            .arg(
                Arg::new(Self::DELETE).long("delete").action(ArgAction::SetTrue).help("Delete the US Cities database."),
            )
    }
    /// Collect the command line arguments and run the migrate command.
    ///
    /// # Arguments
    ///
    /// * `admin_api` is the backend weather administration `API`.
    /// * `args` is the migrate command arguments.
    ///
    pub fn run(admin_api: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
        let cmd_args = Self(args);
        if cmd_args.delete() {
            admin_api.uscities_delete()?;
        }
        if let Some(path) = cmd_args.filename() {
            admin_api.uscities_load(path)?;
        }
        if cmd_args.info() || (cmd_args.filename().is_none() && !cmd_args.delete()) {
            let uscities_info = admin_api.uscities_info()?;
            Self::report_info(uscities_info)?;
        }
        Ok(())
    }
    /// Get the delete command argument.
    ///
    fn filename(&self) -> Option<&PathBuf> {
        self.0.get_one::<PathBuf>(Self::LOAD)
    }

    /// Get the delete command argument.
    ///
    fn delete(&self) -> bool {
        self.0.get_flag(Self::DELETE)
    }

    /// Get the information command argument.
    ///
    fn info(&self) -> bool {
        self.0.get_flag(Self::INFO)
    }

    /// Show information about the US Cities database.
    ///
    /// # Arguments
    ///
    /// * `uscities_info` is the detailed information about the database.
    ///
    fn report_info(uscities_info: UsCityDetails) -> cli::Result<()> {
        let mut report = Report::from(rptcols!(
            <=(0), ^, >,
            <=(0), ^, >,
            <=(0), ^, >,
            <=(0), ^, >,
            <=(0), ^, >
        ));
        if uscities_info.db_size == 0 {
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
            let state_cities = uscities_info.state_info;
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
            report.text(rptrow!(=format!("Database size: {}", mbufmt!(uscities_info.db_size))));
        }
        let mut writer = text::get_writer(&None, false)?;
        text::write_strings(&mut writer, report.into_iter())?;
        Ok(())
    }
}
