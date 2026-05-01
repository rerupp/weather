//! The Weather Data Cities administration utility.
//!
use crate::cli::{self, parse_filename};
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::io::Write;
use std::path::PathBuf;
use toolslib::{
    fmt::commafy,
    mbufmt, rptcols, rptdata,
    text::{self, Report},
};
use weather_lib::admin_prelude::{CitiesDetails, CountryDetails, WeatherAdmin};

/// The US cities administration command name.
pub const COMMAND_NAME: &'static str = "cities";

/// The show information argument id.
const SHOW: &'static str = "SHOW";

/// The initialize database argument id.
const INIT: &'static str = "INIT";

/// The load database argument id.
const LOAD: &'static str = "LOAD";

/// The load database argument id.
const RELOAD: &'static str = "RELOAD";

/// The delete database argument id.
const DELETE: &'static str = "DELETE";

/// Get the US Cities administration utility command definition.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Administer the Cities database.")
        .arg(
            Arg::new(SHOW)
                .short('s')
                .long("show")
                .value_name("*COUNTRY*")
                .action(ArgAction::Set)
                .num_args(0..=1)
                .require_equals(true)
                .help("Show details about the Cities database (default)."),
        )
        .arg(
            Arg::new(INIT)
                .short('i')
                .long("init")
                .action(ArgAction::SetTrue)
                .help("Initialize the Cities database schema."),
        )
        .arg(
            Arg::new(DELETE)
                .short('d')
                .long("delete")
                .action(ArgAction::SetTrue)
                .help("Delete the Cities database file."),
        )
        .arg(
            Arg::new(LOAD)
                .short('l')
                .long("load")
                .value_name("FILE")
                .value_parser(parse_filename)
                .action(ArgAction::Set)
                .num_args(1)
                .require_equals(true)
                .help("Load a Simple Maps city database."),
        )
        .arg(
            Arg::new(RELOAD)
                .short('r')
                .long("reload")
                .action(ArgAction::SetTrue)
                .requires(LOAD)
                .help("Reload the Simple Maps city database."),
        )
}

/// Collect the command line arguments and run the cities command.
/// # Arguments
/// * `admin_api` is the weather data administration API.
/// * `args` is the cities command arguments.
pub fn execute(admin_api: &WeatherAdmin, args: ArgMatches) -> cli::Result<()> {
    // show details if there are no options
    let mut is_default = true;
    // delete the database?
    if args.get_flag(DELETE) {
        admin_api.cities_delete()?;
        is_default = false;
    }
    // initialize the database?
    if args.get_flag(INIT) {
        admin_api.cities_init()?;
        is_default = false;
    }
    // load the database?
    if args.contains_id(LOAD) {
        let filename = args.get_one::<PathBuf>(LOAD).unwrap().display().to_string();
        let reload = args.get_flag(RELOAD);
        admin_api.cities_load(filename, reload)?;
        is_default = false;
    }
    if args.contains_id(SHOW) || is_default {
        match admin_api.cities_details()? {
            None => eprintln!("Cities has not been initialized."),
            Some(details) => {
                let filter = args.get_one::<String>(SHOW).map_or(None, |arg| Some(arg.to_string()));
                report_info(details, filter)?
            }
        }
    }
    Ok(())
}

/// Wrap the error failures that can happen writing to the report.
/// # Params
/// * `writer` is the report writer.
/// * `args` are passed to `format!` to generate the content.
macro_rules! write {
    ($writer:ident, $($arg:tt)*) => {
        if let Err(error) = $writer.write(format!($($arg)*).as_bytes()) {
            cli::err!("failed to write report: {}", error)?;
        }
    };
}

/// Show information about the US Cities database.
/// # Arguments
/// * `details` is the detailed information about the database.
/// * `filter` is used to restrict output to matching countries.
fn report_info(details: CitiesDetails, filter: Option<String>) -> cli::Result<()> {
    let mut writer = text::get_writer(&None, false)?;
    let mut total_cities = 0;
    for country_details in details.country_details {
        total_cities += country_details.region_details.iter().map(|cd| cd.city_count).sum::<usize>();
        if let Some(country_filter) = &filter {
            if !include(country_filter, &country_details.name, &country_details.code) {
                continue;
            }
        }
        report_country_details(&mut writer, country_details)?;
        write!(writer, "\n");
    }
    if total_cities > 0 {
        write!(writer, "Total cities available: {}\n", mbufmt!(total_cities));
    }
    write!(writer, "Database size: {}", mbufmt!(details.db_size));
    Ok(())
}

/// Generate a report for some country.
/// # Arguments
/// * `writer` is where report output will be sent.
/// * `country_details` contains the country details.
fn report_country_details(writer: &mut Box<dyn Write>, country_details: CountryDetails) -> cli::Result<()> {
    // create the report
    let mut report = Report::from(rptcols!(
        <=(0), ^, >,
        <=(0), ^, >,
        <=(0), ^, >,
        <=(0), ^, >,
        <=(0), ^, >
    ));

    // set up the row contents
    let details_per_row = 2;
    let new_row = || Vec::with_capacity(3 * details_per_row);

    // add the headers
    let mut row = new_row();
    for _ in 0..details_per_row {
        row.push(rptdata!(_));
        row.push(rptdata!(^ "Region"));
        row.push(rptdata!(^ "Cities"));
    }
    report.header(row);
    report.separator("-");

    // add the details
    let mut city_count = 0;
    let mut row = new_row();
    for (idx, region_details) in country_details.region_details.into_iter().enumerate() {
        row.push(rptdata!(_));
        row.push(rptdata!(format!("{} ({})", region_details.name, region_details.code)));
        row.push(rptdata!(commafy(region_details.city_count)));
        if (idx % details_per_row) == (details_per_row - 1) {
            report.text(row);
            row = new_row();
        }
        city_count += region_details.city_count;
    }
    if row.len() > 0 {
        report.text(row);
    }
    write!(writer, "{}/{} ({} cities)\n", country_details.name, country_details.code, commafy(city_count));
    text::write_strings(writer, report.into_iter())?;
    Ok(())
}

/// Check if a country should be added to the report.
/// # Arguments
/// * `filter` is the country filter.
/// * `name` is the country name.
/// * `code` is the country code.
fn include(filter: &str, name: &str, code: &str) -> bool {
    // convert everything to lowercase
    let filter_lc = filter.to_lowercase();
    let name_lc = name.to_lowercase();
    let code_lc = code.to_lowercase();

    // now you can check if it should be filtered or not
    let filter_parts = filter_lc.split("*").collect::<Vec<_>>();
    match (filter.starts_with("*"), filter.ends_with("*")) {
        (false, false) => filter_lc == name_lc || filter_lc == code_lc,
        (true, true) => {
            // this matches *foobar* and *
            let filter = filter_parts[1];
            match filter.is_empty() {
                true => true,
                false => name_lc.contains(&filter) || code_lc.contains(&filter),
            }
        }
        (true, false) => {
            let filter = filter_parts[1];
            name_lc.ends_with(filter) || code_lc.ends_with(filter)
        }
        (false, true) => {
            let filter = filter_parts[0];
            name_lc.starts_with(filter) || code_lc.starts_with(filter)
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn filter() {
        assert!(super::include("Foo", "foo", "bar"));
        assert!(super::include("Bar", "foo", "bar"));
        assert!(!super::include("foo", "foobar", "bar"));
        assert!(!super::include("bar", "foo", "foobar"));
        assert!(super::include("*", "foo", "bar"));
        assert!(super::include("*oo", "foo", "bar"));
        assert!(super::include("*ar", "foo", "bar"));
        assert!(super::include("fo*", "foo", "bar"));
        assert!(super::include("ba*", "foo", "bar"));
    }
}
