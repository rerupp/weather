//! The show administrative information command.
use crate::cli;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::io::Write;
use toolslib::{
    mbufmt, rptcols, rptrow,
    text::{self, Report},
};
use weather_lib::admin_prelude::{Components, LocationDetails, WeatherAdmin};

/// The label used for the filesys component name.
const FILESYS_COMPONENT: &str = "File Archives";

/// The show components command.
#[derive(Debug)]
pub struct ShowCmd(
    /// The show command arguments
    pub(super) ArgMatches,
);
impl ShowCmd {
    /// The stat sub-command name.
    pub const NAME: &'static str = "show";
    /// The show differences command argument.
    const DETAILS: &'static str = "DETAILS";
    /// The show differences command argument.
    const DIFFS: &'static str = "DIFF";
    /// Get the drop sub-command definition.
    pub fn get() -> Command {
        Command::new(Self::NAME)
            .about("Show information about the weather data backend components.")
            .arg(
                Arg::new(Self::DETAILS)
                    .long("detail")
                    .action(ArgAction::SetTrue)
                    .help("Show information about the weather history components (default)."),
            )
            .arg(
                Arg::new(Self::DIFFS)
                    .long("diff")
                    .action(ArgAction::SetTrue)
                    .help("Show differences between the weather history components."),
            )
    }
    /// Collect the command line arguments and run the stat database sub-command.
    ///
    /// # Arguments
    ///
    /// * `admin_api` is the backend weather administration `API`.
    pub fn run(admin_api: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
        let components = admin_api.components()?;
        let mut writer = text::get_writer(&None, false)?;
        let cmd_args = ShowCmd(args);
        if cmd_args.details() {
            ShowCmd::component_details(&mut writer, &components)?;
        }
        if cmd_args.diff() {
            locations::audit(&mut writer, &components)?;
            histories::audit(&mut writer, &components)?;
        }
        Ok(())
    }
    /// Generate a report about the weather data component details.
    ///
    /// # Arguments
    ///
    /// * `writer` is where the report will be written.
    /// * `components` contains the details about weather data.
    fn component_details(writer: &mut impl Write, components: &Components) -> cli::Result<()> {
        let mut report = Report::from(rptcols!(<, >, >, >));
        report.header(rptrow!(^ "Component Details", ^ "Size", ^ "Locations", ^ "Histories")).separator("-");
        if let Some(db_details) = &components.db_details {
            let size = mbufmt!(db_details.size);
            let locations = mbufmt!(db_details.location_details.len());
            let histories = mbufmt!(db_details.location_details.iter().map(|d| d.histories).sum::<usize>());
            report.text(rptrow!("Database", size, locations, histories));
        } else {
            log::debug!("Weather data has not been initialized to use a database.");
        }
        let fs_details = &components.fs_details;
        let size = mbufmt!(fs_details.size);
        let locations = mbufmt!(fs_details.location_details.len());
        let histories = mbufmt!(fs_details.location_details.iter().map(|d| d.histories).sum::<usize>());
        report.text(rptrow!(FILESYS_COMPONENT, size, locations, histories));
        report.text(rptrow!(_, + "="));
        let total_size = match &components.db_details {
            Some(db_details) => fs_details.size + db_details.size,
            None => fs_details.size,
        };
        report.text(rptrow!("Overall", mbufmt!(total_size)));
        text::write_strings(writer, report.into_iter())?;
        Ok(())
    }
    /// Get the report details command argument.
    fn details(&self) -> bool {
        match self.diff() {
            true => self.0.get_flag(Self::DETAILS),
            false => true,
        }
    }
    /// Get the show differences command argument.
    fn diff(&self) -> bool {
        self.0.get_flag(Self::DIFFS)
    }
}

mod locations {
    /// Isolate the location differences to the module.
    use super::*;

    /// Generate a report about the component differences for location metadata.
    ///
    /// # Arguments
    ///
    /// * `writer` is where the report will be written.
    /// * `components` contains the details about weather data.
    pub(super) fn audit(writer: &mut impl Write, components: &Components) -> cli::Result<()> {
        let mut missing_locations: Vec<MissingLocations> = vec![];
        if let Some(db_details) = &components.db_details {
            let fs_details = &components.fs_details;
            if let Some(aliases) = cmp_locations(&fs_details.location_details, &db_details.location_details) {
                missing_locations.push(MissingLocations::new("Database", aliases));
            }
            if let Some(aliases) = cmp_locations(&db_details.location_details, &fs_details.location_details) {
                missing_locations.push(MissingLocations::new(FILESYS_COMPONENT, aliases));
            }
        }
        if missing_locations.len() > 0 {
            let mut report = Report::from(rptcols!(<, >, >, >));
            report.header(rptrow!(^ "Component", "Missing Locations")).separator("-");
            for missing in missing_locations {
                report.text(rptrow!(missing.component, missing.locations.join(", ")));
            }
            text::write_strings(writer, report.into_iter())?;
        } else {
            log::debug!("There were no location differences.");
        }
        Ok(())
    }

    /// Compares location details to find if there are differences in location metadata.
    ///
    /// # Arguments
    ///
    /// * `lhs` is the location details being verified.
    /// * `rhs` is the reference location details.
    fn cmp_locations<'l>(lhs: &'l Vec<LocationDetails>, rhs: &'l Vec<LocationDetails>) -> Option<Vec<&'l str>> {
        let aliases = lhs
            .iter()
            .filter_map(|lhs_details| match rhs.iter().any(|rhs_details| lhs_details.alias == rhs_details.alias) {
                true => None,
                false => Some(lhs_details.alias.as_str()),
            })
            .collect::<Vec<&str>>();
        match aliases.is_empty() {
            true => None,
            false => Some(aliases),
        }
    }

    /// Used internally to record missing location metadata.
    #[derive(Debug)]
    struct MissingLocations {
        /// The comonent with missing location metadata.
        component: String,
        /// The locations_win that are missing.
        locations: Vec<String>,
    }
    impl MissingLocations {
        /// Create a new instance of missing location metadata.
        ///
        /// # Arguments
        ///
        /// * `component` is the component name.
        /// * `aliases` is a collection of missing location alias names.
        fn new(component: &str, aliases: Vec<&str>) -> Self {
            Self {
                component: component.to_string(),
                locations: aliases.iter().map(|alias| alias.to_string()).collect(),
            }
        }
    }
}

mod histories {
    /// Consolidate the history differences report to this module.
    use super::*;
    use toolslib::fmt::commafy;

    /// Generate a report about the component differences for weather histories.
    ///
    /// # Arguments
    ///
    /// * `writer` is where the report will be written.
    /// * `components` contains the details about component weather data history.
    pub(super) fn audit(writer: &mut impl Write, components: &Components) -> cli::Result<()> {
        let missing_histories = cmp_histories(components);
        if missing_histories.len() > 0 {
            let mut report = Report::from(rptcols!(<, >, >));
            report.header(rptrow!(^ "Component Histories", ^ "Location", ^ "Missing")).separator("-");
            missing_histories.into_iter().for_each(|h| {
                report.text(rptrow!(h.component, h.alias, commafy(h.histories)));
            });
            text::write_strings(writer, report.into_iter())?;
        } else {
            log::debug!("There were no history differences.");
        }
        Ok(())
    }

    /// Compares the components to determine what the difference in histories are.
    ///
    /// # Arguments
    ///
    /// * `components` contains the details about component weather data history.
    fn cmp_histories(components: &Components) -> Vec<MissingHistories> {
        let mut missing_histories: Vec<MissingHistories> = vec![];
        if let Some(db_details) = &components.db_details {
            let fs_details = &components.fs_details;
            for fs_location in &fs_details.location_details {
                match db_details.location_details.iter().find(|db_location| fs_location.alias == db_location.alias) {
                    Some(db_location) => match fs_location.histories.cmp(&db_location.histories) {
                        std::cmp::Ordering::Less => missing_histories.push(MissingHistories {
                            component: FILESYS_COMPONENT.to_string(),
                            alias: fs_location.alias.to_string(),
                            histories: db_location.histories - fs_location.histories,
                        }),
                        std::cmp::Ordering::Equal => (),
                        std::cmp::Ordering::Greater => missing_histories.push(MissingHistories {
                            component: "Database".to_string(),
                            alias: fs_location.alias.to_string(),
                            histories: fs_location.histories - db_location.histories,
                        }),
                    },
                    None => missing_histories.push(MissingHistories {
                        component: "Database".to_string(),
                        alias: fs_location.alias.to_string(),
                        histories: fs_location.histories,
                    }),
                }
            }
        }
        missing_histories
    }

    /// Used internally to track history differences between components.
    #[derive(Debug)]
    struct MissingHistories {
        /// The name of the component.
        component: String,
        /// The location alias name.
        alias: String,
        /// The number of history differences.
        histories: usize,
    }
}
