//! The user command 'qc' to query US Cities location information.
//!
//! usage: qc [CITY][, STATE][ ZIP_CODE]
//! where:
//!     CITY: [*]string[*]
//!     STATE: [*]string[*]
//!     ZIP_CODE: [*]#####[*]
//!

use crate::cli::{self, err, get_writer, reports::list_locations as reports, user::trim_row_end, ReportArgs};
use clap::{Arg, ArgAction, ArgMatches, Command};
use weather_lib::prelude::{CityFilter, WeatherData};

/// The query cities command name.
///
pub const COMMAND_NAME: &'static str = "qc";

const LIMIT: &str = "QUERY_CITIES_LIMIT";

const FILTER: &str = "QUERY_CITIES_FILTER";

/// create the query cities command.
///
pub fn command() -> Command {
    Command::new(COMMAND_NAME)
        .about("Search cities for location information.")
        .args(vec![
            Arg::new(LIMIT)
                .short('l')
                .long("limit")
                .action(ArgAction::Set)
                .value_name("LIMIT")
                .require_equals(true)
                .value_parser(limit_parser)
                .default_value("30")
                .help("Limit the number of cities shown."),
            Arg::new(FILTER)
                .value_name("FILTER")
                .action(ArgAction::Append)
                .help("The optional city filter ([[*]CITY[*]][, [*]STATE[*]] [[*]ZIP[*]])"),
        ])
        .args(ReportArgs::get())
        .group(ReportArgs::arg_group())
}

pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    let query = match args.get_many::<String>(FILTER) {
        None => Default::default(),
        Some(parts) => parts.map(String::from).collect::<Vec<_>>().join(" "),
    };
    let mut filter = QueryParser::parse(&query)?;
    filter.limit = *args.get_one::<usize>(LIMIT).unwrap();
    match weather_data.search_locations(filter) {
        Err(error) => err!("There was an error searching for locations: {:?}", error)?,
        Ok(locations) => match locations.is_empty() {
            true => println!("There were no locations found."),
            false => {
                let report_args = ReportArgs::new(&args);
                let mut writer = get_writer(&report_args)?;
                let report = if report_args.csv() {
                    reports::csv::Report::default().generate(locations)
                } else if report_args.json() {
                    let report = match report_args.pretty() {
                        true => reports::json::Report::pretty_printed(),
                        false => reports::json::Report::default(),
                    };
                    report.generate(locations)
                } else {
                    reports::text::Report::default()
                        .with_skip_alias()
                        .with_title_separator()
                        .generate(&locations)
                        .into_iter()
                        .map(|row| trim_row_end!(row.to_string()))
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                if let Err(error) = writer.write_all(report.as_bytes()) {
                    err!("There was an error writing the locations report: {:?}", error)?;
                }
            }
        },
    }
    Ok(())
}

/// Used by the command parser to make sure the limit is within bounds.
///
/// # Arguments
///
/// * `limit_arg` is the weather directory command argument.
///
fn limit_parser(limit_arg: &str) -> Result<usize, String> {
    match limit_arg.parse::<usize>() {
        Ok(limit) => match limit > 0 {
            true => Ok(limit),
            _ => Err("limit must be greater than 0".to_string()),
        },
        Err(_) => Err("limit needs to be an unsigned integer.".to_string()),
    }
}

/// The parser that mines the city, state, and zip parts of query.
///
/// A full query takes the form of: CITY, STATE ZIP
///
/// The parse rules:
///
/// * a comma separates the CITY from the STATE and/or ZIP
/// * the STATE follows a comma and ending at the first digit
/// * the ZIP starts at the first digit
///
#[derive(Debug, Default)]
struct QueryParser {
    filter: CityFilter,
}
impl QueryParser {
    fn parse(query: &str) -> cli::Result<CityFilter> {
        if query.chars().find(|c| *c == ',').iter().count() > 1 {
            Err(cli::Error::from("Only 1 comma can be used in the query."))?;
        }
        let mut self_ = QueryParser::default();
        let city_end_idx = self_.parse_name(query);
        let state_end_idx = city_end_idx + self_.parse_state(&query[city_end_idx..]);
        self_.parse_zip(&query[state_end_idx..]);
        Ok(self_.filter)
    }
    fn parse_name(&mut self, query: &str) -> usize {
        // walk the string until you find a separator
        let mut end_idx = 0usize;
        for char in query.chars() {
            if char == ',' || char.is_digit(10) {
                break;
            }
            end_idx += 1;
        }

        // save the city name
        if end_idx > 0 {
            self.filter.name.replace(query[..end_idx].trim().to_string());
        }

        // send back the idx of the last
        end_idx
    }
    fn parse_state(&mut self, query: &str) -> usize {
        let mut start_idx = 0usize;
        let mut end_idx = start_idx;
        for char in query.chars() {
            if char.is_digit(10) {
                break;
            }
            // the char can only be a comma on the first pass
            if char != ',' {
                end_idx += 1;
            } else {
                start_idx += 1;
                end_idx = start_idx;
            }
        }
        if end_idx > start_idx {
            self.filter.state.replace(query[start_idx..end_idx].trim().to_string());
        }
        end_idx
    }
    fn parse_zip(&mut self, query: &str) {
        if query.len() > 0 {
            self.filter.zip_code.replace(query.trim().to_string());
        }
    }
}
