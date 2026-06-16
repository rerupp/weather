//! The weather data user CLI commands.
use crate::cli;
use chrono::NaiveDate;
use clap::{ArgMatches, Command};
use weather_lib::prelude::WeatherData;

mod add_history;
mod add_location;
mod delete_location;
mod list_cities;
mod list_history;
mod list_locations;
mod list_summary;
mod location;
mod modify_location;
mod report_history;

#[derive(Debug)]
pub struct User;
impl User {
    /// Return the collection of user commands.
    pub fn get_commands() -> Vec<Command> {
        vec![
            add_location::command(),
            modify_location::command(),
            delete_location::command(),
            list_locations::command(),
            list_history::command(),
            list_summary::command(),
            add_history::command(),
            report_history::command(),
            list_cities::command(),
        ]
    }
    /// Run the associated command.
    ///
    /// # Arguments
    ///
    /// - `weather_data` is the weather history API that will be used.
    /// - `name` identifies the command that will be run.
    /// - `args` holds the associated command arguments.
    pub fn run(weather_data: &WeatherData, name: &str, args: ArgMatches) -> cli::Result<()> {
        match name {
            add_location::COMMAND_NAME => add_location::execute(weather_data, args),
            modify_location::COMMAND_NAME => modify_location::execute(weather_data, args),
            delete_location::COMMAND_NAME => delete_location::execute(weather_data, args),
            list_locations::COMMAND_NAME => list_locations::execute(weather_data, args),
            list_history::COMMAND_NAME => list_history::execute(weather_data, args),
            list_summary::COMMAND_NAME => list_summary::execute(weather_data, args),
            add_history::COMMAND_NAME => add_history::execute(weather_data, args),
            report_history::COMMAND_NAME => report_history::execute(weather_data, args),
            list_cities::COMMAND_NAME => list_cities::execute(weather_data, args),
            _ => unreachable!("User command should not be here..."),
        }
    }
}

/// Validate the location argument to make sure it's not missing.
///
/// # Arguments
///
/// * `name` is the command line argument that should be a location name.
fn validate_location(name: &str) -> Result<String, String> {
    match toolslib::date_time::parse_date(name) {
        Ok(_) => Err("The location name is a date.".to_string()),
        Err(_) => Ok(name.to_string()),
    }
}

/// Trim trailing whitespace from the string.
///
/// # Params
///
/// * `string` is what will be trimmed.
///
macro_rules! trim_row_end {
    ($string:expr) => {
        $string.trim_end().to_string()
    };
}
use trim_row_end;

mod location_filters {
    /// The CLI location filter query argument and parser.
    ///
    use super::cli;
    use clap::{Arg, ArgAction, ArgMatches};
    use weather_lib::prelude::LocationFilter;

    /// The list search filter argument id.
    ///
    const FILTER: &str = "FILTER";

    /// The multiple location name separator.
    ///
    const MULTI_MARKER: char = '+';

    /// The region name separator.
    ///
    const REGION_MARKER: char = ',';

    /// The country name separator.
    ///
    const COUNTRY_MARKER: char = '@';

    /// The location filters string syntax.
    ///
    pub const FILTER_SYNTAX: &str = "[[*]NAME[*]+...][, [*]REGION[*]] [@ [*]COUNTRY[*]]";

    /// Get a copy of the filter argument.
    ///
    pub fn arg() -> Arg {
        Arg::new(FILTER)
            .value_name(FILTER)
            .action(ArgAction::Append)
            // .help("The optional location filter ([[*]NAME[*]+...][, [*]REGION[*]] [@ [*]COUNTRY[*]])")
            .help(format!("The optional location filter ({FILTER_SYNTAX})"))
    }

    pub fn get_query_str(arg_matches: &ArgMatches) -> Option<String> {
        match arg_matches.get_many::<String>(FILTER) {
            Some(args) => Some(args.map(String::from).collect::<Vec<_>>().join(" ")),
            None => None,
        }
    }

    /// The parser that creates a [LocationFilter] from the command line arguments.
    ///
    /// # Arguments
    ///
    /// * `arg_matches` contains the filter command line arguments.
    ///
    pub fn parse_args(arg_matches: &ArgMatches) -> cli::Result<Option<Vec<LocationFilter>>> {
        match get_query_str(arg_matches) {
            Some(query) => parse(query),
            None => Ok(None),
        }
    }

    /// The parser that creates a [LocationFilter] collection from a query string.
    ///
    /// # Arguments
    ///
    /// * `query_str` is what will be converted into a collection of [LocationFilter].
    ///
    pub fn parse(query_str: impl ToString) -> cli::Result<Option<Vec<LocationFilter>>> {
        // bail if there are no filters
        let query_str = query_str.to_string();
        let mut query = query_str.trim();
        if query.is_empty() {
            return Ok(None);
        }

        // get the marker counts
        let region_markers = query.chars().filter(|c| *c == REGION_MARKER).count();
        if region_markers > 1 {
            cli::err!("Only 1 region can be used in a query.")?;
        }
        let country_markers = query.chars().filter(|c| *c == COUNTRY_MARKER).count();
        if country_markers > 1 {
            cli::err!("Only 1 country can be used in a query.")?;
        }

        // make sure the country comes after the region
        if region_markers > 0 && country_markers > 0 {
            let comma_idx = query.find(REGION_MARKER).unwrap();
            let plus_idx = query.find(COUNTRY_MARKER).unwrap();
            if comma_idx > plus_idx {
                cli::err!("The country cannot come before the region.")?;
            }
        }

        // check if there are multi markers
        if query.chars().filter(|c| *c == MULTI_MARKER).count() > 0 {

            // make sure multi marker isn't used on the region or country
            if region_markers > 0 || country_markers > 0 {

                // look at the filter right to left so that country and region are prior to name
                let reversed_filters = query.chars().rev().into_iter().collect::<String>();
                let multi_index = reversed_filters.find(MULTI_MARKER).unwrap();

                // check the country first
                if country_markers > 0 {
                    let marker_idx = reversed_filters.find(COUNTRY_MARKER).unwrap();
                    if multi_index < marker_idx {
                        cli::err!("Selecting multiple countries is not allowed.")?;
                    }
                }

                // now region marker
                if region_markers > 0 {
                    let marker_idx = reversed_filters.find(REGION_MARKER).unwrap();
                    if multi_index < marker_idx {
                        cli::err!("Selecting multiple regions is not allowed.")?;
                    }
                }
            }
        }

        macro_rules! parse_opt {
            ($marker_count: ident, $marker: ident) => {{
                let mut filter_opt = None;
                if $marker_count > 0 {
                    // break the filters into 2 parts
                    let mut query_parts = query.split($marker).collect::<Vec<_>>();
                    // only update the filter if it is not empty
                    let filter = query_parts.pop().unwrap().trim();
                    if !filter.is_empty() {
                        filter_opt.replace(filter.to_string());
                    }
                    // update the filters with what is left
                    query = query_parts.pop().unwrap().trim();
                }
                filter_opt
            }};
        }

        // parse the filters right to left
        let country_opt = parse_opt!(country_markers, COUNTRY_MARKER);
        let region_opt = parse_opt!(region_markers, REGION_MARKER);

        // parse the location names
        let mut location_filters = vec![];
        if !query.is_empty() {
            for name in query.split(MULTI_MARKER).collect::<Vec<&str>>() {
                // only add a filter if there is a name available
                let name = name.trim();
                if !name.is_empty() {
                    let filter = LocationFilter {
                        alias: Some(name.to_string()),
                        city: Some(name.to_string()),
                        region: region_opt.clone(),
                        country: country_opt.clone(),
                    };
                    location_filters.push(filter);
                }
            }
        }

        // add a region or country filter if there were no location names
        if location_filters.is_empty() && (country_opt.is_some() || region_opt.is_some()) {
            let filter = LocationFilter { region: region_opt, country: country_opt, ..Default::default() };
            location_filters.push(filter);
        }

        match location_filters.is_empty() {
            true => Ok(None),
            false => Ok(Some(location_filters)),
        }
    }

    #[cfg(test)]
    mod tests {
        use clap::Command;
        use weather_lib::prelude::LocationFilter;

        macro_rules! filter_eq {
            ($lhs: expr, ($rhs_name: literal, $rhs_region: literal, $rhs_country: literal)) => {{
                let mut rhs = LocationFilter::default();
                if !$rhs_name.is_empty() {
                    rhs.alias.replace($rhs_name.to_string());
                    rhs.city.replace($rhs_name.to_string());
                }
                if !$rhs_region.is_empty() {
                    rhs.region.replace($rhs_region.to_string());
                }
                if !$rhs_country.is_empty() {
                    rhs.country.replace($rhs_country.to_string());
                }
                assert_eq!($lhs.alias, rhs.alias, "alias did not match");
                assert_eq!($lhs.city, rhs.city, "city did not match");
                assert_eq!($lhs.region, rhs.region, "region did not match");
                assert_eq!($lhs.country, rhs.country, "country did not match");
            }};
        }

        #[test]
        fn parse_args() {
            let parse = |args: Vec<&str>|
                super::parse_args(
                    &mut Command::new("test").no_binary_name(true).arg(super::arg()).get_matches_from(args),
                );


            let testcase = parse(vec![" name "]).unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "", ""));

            let testcase = parse(vec!["", ",", " region "]).unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("", "region", ""));

            let testcase = parse(vec!["", ",", "", "@", " country "]).unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("", "", "country"));

            let testcase = parse(vec!["name1", "+", "name2", ",", "region", "@", "country"]).unwrap().unwrap();
            assert_eq!(testcase.len(), 2);
            filter_eq!(testcase[0], ("name1", "region", "country"));
            filter_eq!(testcase[1], ("name2", "region", "country"));
        }

        #[test]
        fn parse() {
            macro_rules! parse {
                ($filter: literal) => {
                    super::parse($filter.to_string())
                };
            }

            // error conditions
            assert!(parse!(",,").is_err());
            assert!(parse!("@@").is_err());
            assert!(parse!("@,").is_err());
            assert!(parse!(",+").is_err());
            assert!(parse!("@+").is_err());
            assert!(parse!(",+@+").is_err());

            // default filters
            assert!(parse!("").unwrap().is_none());
            assert!(parse!("+").unwrap().is_none());
            assert!(parse!(",").unwrap().is_none());
            assert!(parse!("@").unwrap().is_none());
            assert!(parse!("+,@").unwrap().is_none());

            let testcase = parse!(" name ").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "", ""));
            let testcase = parse!("name+").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "", ""));
            let testcase = parse!("name,").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "", ""));
            let testcase = parse!("name@").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "", ""));
            let testcase = parse!("name,@").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "", ""));
            let testcase = parse!("name1 + name2,@").unwrap().unwrap();
            assert_eq!(testcase.len(), 2);
            filter_eq!(testcase[0], ("name1", "", ""));
            filter_eq!(testcase[1], ("name2", "", ""));

            let testcase = parse!(", region ").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("", "region", ""));
            let testcase = parse!("name,region").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "region", ""));
            let testcase = parse!("name, region @").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "region", ""));
            let testcase = parse!("name1+name2,region").unwrap().unwrap();
            assert_eq!(testcase.len(), 2);
            filter_eq!(testcase[0], ("name1", "region", ""));
            filter_eq!(testcase[1], ("name2", "region", ""));

            let testcase = parse!("@ country ").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("", "", "country"));
            let testcase = parse!("name @ country").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "", "country"));
            let testcase = parse!("name,@country").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "", "country"));
            let testcase = parse!(" name , region @ country ").unwrap().unwrap();
            assert_eq!(testcase.len(), 1);
            filter_eq!(testcase[0], ("name", "region", "country"));
            let testcase = parse!("name1+name2,region@country").unwrap().unwrap();
            assert_eq!(testcase.len(), 2);
            filter_eq!(testcase[0], ("name1", "region", "country"));
            filter_eq!(testcase[1], ("name2", "region", "country"));
        }
    }
}

mod date_arg {
    //! This module provides utilities for the command line arguments that are dates.
    //!
    //! The add history and report history cli commands use dates. The [try_parse] command
    //! supports the following date formats.
    //!
    //!     YYYY
    //!     MM-YYYY MM/YYYY
    //!     YYYY-MM YYYY/MM
    //!     MMM-YYYY
    //!     MM-DD-YYYY MM/DD/YYYY
    //!     YYYY-MM-DD YYYY/MM/DD
    //!
    use super::*;
    use chrono::{Datelike, Days, Months};
    use regex::Regex;
    use std::sync::LazyLock;
    use weather_lib::prelude::DateRange;

    /// Test if the date string appears to be a year.
    ///
    /// # Arguments
    ///
    /// * `date` is the string that will be examined.
    ///
    pub fn is_year(date: &str) -> bool {
        YEAR_RE.is_match(date)
    }

    /// Test if the date string appears to be a month and year.
    ///
    /// # Arguments
    ///
    /// * `date` is the string that will be examined.
    ///
    pub fn is_month_year(date: &str) -> bool {
        TEXT_MONTH_YEAR_RE.is_match(date) || MONTH_YEAR_RE.is_match(date) || YEAR_MONTH_RE.is_match(date)
    }

    /// Create a new date that is the last day of the year.
    ///
    /// # Arguments
    ///
    /// * `date` identifies the year of the last day.
    ///
    pub fn to_eoy(date: &NaiveDate) -> NaiveDate {
        NaiveDate::from_ymd_opt(date.year(), 12, 31).unwrap()
    }

    /// Create a new date that is the last day of the month.
    ///
    /// # Arguments
    ///
    /// * `date` identifies the month and year of the last day.
    ///
    pub fn to_eom(date: &NaiveDate) -> NaiveDate {
        NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
            .unwrap()
            .checked_add_months(Months::new(1))
            .unwrap()
            .checked_sub_days(Days::new(1))
            .unwrap()
    }

    /// Parse a start and end date into a [DateRange].
    ///
    /// If the end argument is missing the end of the date range will be calculated using the following
    /// rules.
    ///
    /// * if the start argument is a month and year, the end date will be set to the last day of the month.
    /// * if the start argument is a year, the end date will be set to the last day of the year.
    /// * otherwise the end date will be the start date
    ///
    /// # Arguments
    ///
    /// * `start_arg` is the start date argument.
    /// * `end_arg` is the optional end date argument.
    ///
    pub fn try_parse_daterange(start_arg: &str, end_arg: Option<&str>) -> cli::Result<DateRange> {
        let start = match try_parse(start_arg) {
            Ok(date) => date,
            Err(parse_error) => cli::err!("{parse_error}")?,
        };
        let end = match end_arg {
            None => {
                if is_year(start_arg) {
                    to_eoy(&start)
                } else if is_month_year(start_arg) {
                    to_eom(&start)
                } else {
                    start.clone()
                }
            }
            Some(end_arg) => match try_parse(end_arg) {
                Err(parse_error) => cli::err!("{parse_error}")?,
                Ok(date) => {
                    if is_year(end_arg) {
                        to_eoy(&date)
                    } else if is_month_year(end_arg) {
                        to_eom(&date)
                    } else {
                        date
                    }
                }
            },
        };
        if start > end {
            cli::err!("The end date is before the start date.")?;
        }
        Ok(DateRange::new(start, end))
    }

    /// Parse a date string into a date.
    ///
    /// # Arguments
    ///
    /// * `date_arg` is the date string that will be parsed.
    ///
    pub fn try_parse(date_arg: &str) -> Result<NaiveDate, String> {
        if let Some(parsed_date) = try_year(date_arg) {
            Ok(parsed_date)
        } else if let Some(parse_result) = try_month_year(date_arg) {
            parse_result
        } else if let Some(parse_result) = try_text_month_year(date_arg) {
            parse_result
        } else if let Some(parse_result) = try_full_date(date_arg) {
            parse_result
        } else {
            Err(format!("Could not parse '{date_arg}'."))
        }
    }

    /// The regular expression that matches a year pattern.
    ///
    static YEAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?<y>\d{4})$").unwrap());

    /// Try to parse the string as a year only argument.
    ///
    /// # Arguments
    ///
    /// * `date_arg` is the string that will be parsed.
    ///
    fn try_year(date_arg: &str) -> Option<NaiveDate> {
        let mut parsed_date = None;
        if let Some(cap) = YEAR_RE.captures(date_arg) {
            let year = cap["y"].parse::<i32>().unwrap();
            parsed_date.replace(NaiveDate::from_ymd_opt(year, 1, 1).unwrap());
        };
        parsed_date
    }

    /// The regular expression that matches the abbreviated month and year pattern.
    ///
    static TEXT_MONTH_YEAR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(?<m>[A-Za-z]{3})[-/](?<y>\d{4})$").unwrap());

    /// Try to parse the string as an abbreviated month and year argument.
    ///
    /// # Arguments
    ///
    /// * `date_arg` is the string that will be parsed.
    ///
    fn try_text_month_year(date_arg: &str) -> Option<Result<NaiveDate, String>> {
        let mut parsed_date = None;
        if let Some(cap) = TEXT_MONTH_YEAR_RE.captures(date_arg) {
            let name = &cap["m"];
            match try_parse_month(name) {
                Err(error) => {
                    parsed_date.replace(Err(error));
                }
                Ok(month) => {
                    let year = cap["y"].parse::<i32>().unwrap();
                    parsed_date.replace(Ok(NaiveDate::from_ymd_opt(year, month, 1).unwrap()));
                }
            }
        }
        parsed_date
    }

    /// Get the numeric month from either a short or full month name.
    ///
    /// # Arguments
    ///
    /// * `name` is the month name that will be parsed.
    ///
    fn try_parse_month(name: &str) -> Result<u32, String> {
        match name.to_lowercase().as_str() {
            "jan" | "january" => Ok(1),
            "feb" | "february" => Ok(2),
            "mar" | "march" => Ok(3),
            "apr" | "april" => Ok(4),
            "may" => Ok(5),
            "jun" | "june" => Ok(6),
            "jul" | "july" => Ok(7),
            "aug" | "august" => Ok(8),
            "sep" | "september" => Ok(9),
            "oct" | "october" => Ok(10),
            "nov" | "november" => Ok(11),
            "dec" | "december" => Ok(12),
            _ => Err(format!("'{name}' is not a valid month name.")),
        }
    }

    /// The regular expression that matches the numeric month and year pattern.
    ///
    static MONTH_YEAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?<m>\d{2})[-/](?<y>\d{4})$").unwrap());

    /// The regular expression that matches the numeric year and month pattern.
    ///
    static YEAR_MONTH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?<y>\d{4})[-/](?<d>\d{2})$").unwrap());

    /// Try to parse the string as a numeric month and year argument.
    ///
    /// # Arguments
    ///
    /// * `date_arg` is the string that will be parsed.
    ///
    fn try_month_year(date_arg: &str) -> Option<Result<NaiveDate, String>> {
        let mut captures = MONTH_YEAR_RE.captures(date_arg);
        if captures.is_none() {
            captures = YEAR_MONTH_RE.captures(date_arg);
        }
        let mut parsed_date = None;
        if let Some(cap) = captures {
            let month = cap["m"].parse::<u32>().unwrap();
            if month < 1 || month > 12 {
                parsed_date.replace(Err("The month is out of bounds".to_string()));
            } else {
                let year = cap["y"].parse::<i32>().unwrap();
                parsed_date.replace(Ok(NaiveDate::from_ymd_opt(year, month, 1).unwrap()));
            }
        }
        parsed_date
    }

    /// The regular expression that matches the numeric month, day, year pattern.
    ///
    static MONTH_DAY_YEAR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(?<m>\d{2})[-/](?<d>\d{2})[-/](?<y>\d{4})$").unwrap());

    /// The regular expression that matches the numeric year, month, day pattern.
    ///
    static YEAR_MONTH_DAY_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(?<y>\d{4})[-/](?<m>\d{2})[-/](?<d>\d{2})$").unwrap());

    /// The regular expression that matches the text month, day. year pattern
    ///
    static TEXT_MONTH_DAY_YEAR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(?<m>[A-Za-z]{3,})[-/](?<d>\d{2})[-/](?<y>\d{4})$").unwrap());

    /// Try to parse the string as a numeric day, month and year argument.
    ///
    /// # Arguments
    ///
    /// * `date_arg` is the string that will be parsed.
    ///
    fn try_full_date(date: &str) -> Option<Result<NaiveDate, String>> {
        let mut text_month = false;
        let mut captures = MONTH_DAY_YEAR_RE.captures(date);
        if captures.is_none() {
            captures = YEAR_MONTH_DAY_RE.captures(date);
            if captures.is_none() {
                text_month = true;
                captures = TEXT_MONTH_DAY_YEAR_RE.captures(date);
            }
        }
        let mut parsed_date = None;
        if let Some(cap) = captures {
            let month = match text_month {
                true => match try_parse_month(&cap["m"]) {
                    Ok(month) => Some(month),
                    Err(error) => {
                        parsed_date.replace(Err(error));
                        None
                    }
                },
                false => {
                    let month = cap["m"].parse::<u32>().unwrap();
                    if month >= 1 && month <= 12 {
                        Some(month)
                    } else {
                        parsed_date.replace(Err("The dates month is out of bounds.".to_string()));
                        None
                    }
                }
            };
            if let Some(month) = month {
                let year = cap["y"].parse::<i32>().unwrap();
                let day = cap["d"].parse::<u32>().unwrap();
                let result = match NaiveDate::from_ymd_opt(year, month, day) {
                    Some(d) => Ok(d),
                    None => Err(format!("'{date}' is not valid.")),
                };
                parsed_date.replace(result);
            }
        }
        parsed_date
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        macro_rules! date {
            ($y: literal, $m: literal, $d: literal) => {
                NaiveDate::from_ymd_opt($y, $m, $d).unwrap()
            };
        }

        #[test]
        fn parse_month() {
            macro_rules! test_parse {
                ($short: literal, $long: literal, $month: literal) => {
                    assert_eq!(try_parse_month($short), Ok($month), "failed to parse '{}'", $short);
                    assert_eq!(try_parse_month($long), Ok($month), "failed to parse '{}'", $long);
                };
            }
            test_parse!("jan", "january", 1);
            test_parse!("feb", "february", 2);
            test_parse!("mar", "march", 3);
            test_parse!("apr", "april", 4);
            test_parse!("May", "MAY", 5);
            test_parse!("jun", "june", 6);
            test_parse!("jul", "july", 7);
            test_parse!("aug", "august", 8);
            test_parse!("sep", "september", 9);
            test_parse!("oct", "october", 10);
            test_parse!("nov", "november", 11);
            test_parse!("dec", "december", 12);
        }

        #[test]
        fn parse_daterange() {
            macro_rules! testcase {
                ($dr_start: expr, $dr_end: expr, $start: expr, $end: expr) => {
                    let testcase = try_parse_daterange($dr_start, $dr_end).unwrap();
                    assert_eq!(testcase.start, $start, "{} start date", testcase);
                    assert_eq!(testcase.end, $end, "{} end date", testcase);
                };
            }
            testcase!("2025", None, date!(2025, 1, 1), date!(2025, 12, 31));
            testcase!("2025", Some("2025"), date!(2025, 1, 1), date!(2025, 12, 31));
            testcase!("mar-2025", None, date!(2025, 3, 1), date!(2025, 3, 31));
            testcase!("mar-2025", Some("mar-2025"), date!(2025, 3, 1), date!(2025, 3, 31));
            testcase!("03-01-2025", None, date!(2025, 3, 1), date!(2025, 3, 1));
            testcase!("2024", Some("2025"), date!(2024, 1, 1), date!(2025, 12, 31));
            testcase!("oct-2025", Some("nov-2025"), date!(2025, 10, 1), date!(2025, 11, 30));
            testcase!("05-01-2025", None, date!(2025, 5, 1), date!(2025, 5, 1));
            testcase!("06-02-2025", Some("06-15-2025"), date!(2025, 6, 2), date!(2025, 6, 15));
            testcase!("03-2025", Some("2025"), date!(2025, 3, 1), date!(2025, 12, 31));
            testcase!("mar-10-2025", Some("april-25-2025"), date!(2025, 3, 10), date!(2025, 4, 25));
        }
        #[test]
        fn arg_types() {
            assert!(is_year("2025"));
            assert!(!is_year("11-2025"));
            assert!(is_month_year("11-2025"));
            assert!(is_month_year("2025-11"));
            assert!(is_month_year("nov-2025"));
            assert!(!is_month_year("2025"));
            assert!(!is_month_year("11-15-2025"));
        }
        #[test]
        fn eom() {
            let testcase = to_eom(&date!(2023, 2, 14));
            assert_eq!(testcase.year(), 2023);
            assert_eq!(testcase.month(), 2);
            assert_eq!(testcase.day(), 28);
            let testcase = to_eom(&date!(2024, 2, 1));
            assert_eq!(testcase.year(), 2024);
            assert_eq!(testcase.month(), 2);
            assert_eq!(testcase.day(), 29);
        }
        #[test]
        fn eoy() {
            let testcase = to_eoy(&date!(1970, 1, 1));
            assert_eq!(testcase.year(), 1970);
            assert_eq!(testcase.month(), 12);
            assert_eq!(testcase.day(), 31);
        }
        #[test]
        fn parse() {
            assert_eq!(try_parse("2025").unwrap(), date!(2025, 1, 1));
            assert_eq!(try_parse("02-2025").unwrap(), date!(2025, 2, 1));
            assert_eq!(try_parse("Mar-2025").unwrap(), date!(2025, 3, 1));
            assert_eq!(try_parse("04-15-2025").unwrap(), date!(2025, 4, 15));
            assert!(try_parse("June-2025").is_err());
        }
        #[test]
        fn yyyy() {
            assert!(try_year("1").is_none());
            assert!(try_year("19").is_none());
            assert!(try_year("197").is_none());
            assert_eq!(try_year("1970").unwrap(), date!(1970, 1, 1));
            assert!(try_year("19700").is_none());
        }
        #[test]
        fn text_month_year() {
            assert!(try_text_month_year("123-1970").is_none());
            assert!(try_text_month_year("J-1970").is_none());
            assert!(try_text_month_year("JA-1970").is_none());
            let epoch = date!(1970, 1, 1);
            assert_eq!(try_text_month_year("jan-1970"), Some(Ok(epoch)));
            assert_eq!(try_text_month_year("jan/1970"), Some(Ok(epoch)));
            assert_eq!(try_text_month_year("JAN-1970"), Some(Ok(epoch)));
            assert_eq!(try_text_month_year("Jan-1970"), Some(Ok(epoch)));
            assert_eq!(try_text_month_year("jAn-1970"), Some(Ok(epoch)));
            assert_eq!(try_text_month_year("jaN-1970"), Some(Ok(epoch)));
            assert!(try_text_month_year("feb-1970").unwrap().is_ok());
            assert!(try_text_month_year("mar-1970").unwrap().is_ok());
            assert!(try_text_month_year("apr-1970").unwrap().is_ok());
            assert!(try_text_month_year("may-1970").unwrap().is_ok());
            assert!(try_text_month_year("jun-1970").unwrap().is_ok());
            assert!(try_text_month_year("jul-1970").unwrap().is_ok());
            assert!(try_text_month_year("aug-1970").unwrap().is_ok());
            assert!(try_text_month_year("sep-1970").unwrap().is_ok());
            assert!(try_text_month_year("oct-1970").unwrap().is_ok());
            assert!(try_text_month_year("nov-1970").unwrap().is_ok());
            assert!(try_text_month_year("dec-1970").unwrap().is_ok());
            assert!(try_text_month_year("abc-1970").unwrap().is_err());
        }
        #[test]
        fn month_year() {
            assert!(try_month_year("1-1970").is_none());
            assert!(try_month_year("01-197").is_none());
            let epoch = date!(1970, 1, 1);
            assert_eq!(try_month_year("01-1970"), Some(Ok(epoch)));
            assert_eq!(try_month_year("01/1970"), Some(Ok(epoch)));
            assert_eq!(try_month_year("01-1970"), Some(Ok(epoch)));
            assert_eq!(try_month_year("01/1970"), Some(Ok(epoch)));
            assert!(try_month_year("00-0000").unwrap().is_err());
            assert!(try_month_year("01-0000").unwrap().is_ok());
            assert!(try_month_year("01-9999").unwrap().is_ok());
        }
        #[test]
        fn full_date() {
            assert!(try_full_date("970-01-01").is_none());
            assert!(try_full_date("1970-1-01").is_none());
            assert!(try_full_date("1970-01-1").is_none());
            let epoch = date!(1970, 1, 1);
            assert_eq!(try_full_date("1970-01-01"), Some(Ok(epoch)));
            assert_eq!(try_full_date("1970/01/01"), Some(Ok(epoch)));
            assert_eq!(try_full_date("01-01-1970"), Some(Ok(epoch)));
            assert_eq!(try_full_date("01/01/1970"), Some(Ok(epoch)));
            assert!(try_full_date("1970/00/01").unwrap().is_err());
            assert!(try_full_date("1970/13/01").unwrap().is_err());
            assert!(try_full_date("1970/01/00").unwrap().is_err());
            assert!(try_full_date("1970/01/32").unwrap().is_err());
            assert!(try_full_date("2024/02/29").unwrap().is_ok());
            assert!(try_full_date("2025/02/29").unwrap().is_err());
            assert!(try_full_date("01-01-197").is_none());
            assert!(try_full_date("1-01-1970").is_none());
            assert!(try_full_date("01-1-1970").is_none());
            assert!(try_full_date("02/29/2024").unwrap().is_ok());
            assert!(try_full_date("02/29/2025").unwrap().is_err());
            assert_eq!(try_full_date("Jan-01-2025").unwrap().unwrap(), date!(2025, 1, 1));
            assert_eq!(try_full_date("January-31-2025").unwrap().unwrap(), date!(2025, 1, 31));
            assert_eq!(try_full_date("may-01-2025").unwrap().unwrap(), date!(2025, 5, 1));
        }
    }
}
