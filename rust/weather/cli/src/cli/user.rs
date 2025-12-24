//! The weather data user CLI commands.
use crate::cli;
use chrono::NaiveDate;
use clap::{ArgMatches, Command};
use weather_lib::prelude::{Location, LocationFilter, WeatherData};

mod add_history;
mod add_location;
mod delete_location;
mod list_history;
mod list_locations;
mod list_summary;
mod modify_location;
mod query_cities;
mod query_states;
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
            query_cities::command(),
            query_states::command(),
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
            query_cities::COMMAND_NAME => query_cities::execute(weather_data, args),
            query_states::COMMAND_NAME => query_states::execute(weather_data, args),
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
macro_rules! trim_row_end {
    ($string:expr) => {
        $string.trim_end().to_string()
    };
}
use trim_row_end;

/// Get a location by name.
///
/// # Arguments
///
/// * `weather_data` is the weather history data `API`.
/// * `name` identifies which location should be returned.
/// * `multi_action` is called if there are multiple locations.
///
fn get_location<F>(weather_data: &WeatherData, name: &str, on_multiple: F) -> cli::Result<Option<Location>>
where
    F: FnOnce() -> cli::Result<()>,
{
    match weather_data.get_locations(Some(vec![LocationFilter::name(name)])) {
        Err(error) => cli::err!("Error getting the location: {error}."),
        Ok(mut locations) => match locations.len() {
            0 => Ok(None),
            1 => Ok(locations.pop()),
            _ => {
                on_multiple()?;
                Ok(None)
            }
        },
    }
}
mod location_args {
    use super::*;
    use clap::{Arg, ArgAction};

    /// The common command line arguments.
    pub struct LocationArgs<'a>(
        /// The subcommand command line arguments.
        &'a ArgMatches,
    );
    impl<'a> LocationArgs<'a> {
        /// The command argument id for the city name.
        const CITY_NAME: &'static str = "CITY_NAME";

        /// The command argument id for the state ID.
        const STATE_ID: &'static str = "STATE_ID";

        /// The command argument id for the state name.
        const STATE: &'static str = "STATE";

        /// The command argument id for the latitude.
        const LATITUDE: &'static str = "LATITUDE";

        /// The command argument id for the longitude.
        const LONGITUDE: &'static str = "LONGITUDE";

        /// The command argument id for the longitude.
        const TZ: &'static str = "TZ";

        /// Get the location attribute arguments.
        pub fn get(required: bool) -> Vec<Arg> {
            vec![
                Arg::new(Self::CITY_NAME)
                    .long("city")
                    .require_equals(true)
                    .value_name("CITY")
                    .required(required)
                    .action(ArgAction::Set)
                    .help("The location city name."),
                Arg::new(Self::STATE_ID)
                    .long("id")
                    .require_equals(true)
                    .value_name("ID")
                    .required(required)
                    .action(ArgAction::Set)
                    .help("The location abbreviated state name."),
                Arg::new(Self::STATE)
                    .long("state")
                    .require_equals(true)
                    .value_name("STATE")
                    .required(required)
                    .action(ArgAction::Set)
                    .help("The location state name."),
                Arg::new(Self::LATITUDE)
                    .long("lat")
                    .require_equals(true)
                    .value_name("LATITUDE")
                    .action(ArgAction::Set)
                    .required(required)
                    .help("The location latitude."),
                Arg::new(Self::LONGITUDE)
                    .long("lon")
                    .require_equals(true)
                    .value_name("LONGITUDE")
                    .required(required)
                    .action(ArgAction::Set)
                    .help("The location longitude."),
                Arg::new(Self::TZ)
                    .long("tz")
                    .require_equals(true)
                    .value_name("TZ")
                    .required(required)
                    .action(ArgAction::Set)
                    .help("The location timezone."),
            ]
        }
        pub fn is_none(&self) -> bool {
            !(self.0.contains_id(Self::CITY_NAME)
                || self.0.contains_id(Self::STATE_ID)
                || self.0.contains_id(Self::STATE)
                || self.0.contains_id(Self::LATITUDE)
                || self.0.contains_id(Self::LONGITUDE)
                || self.0.contains_id(Self::TZ))
        }
        pub fn city(&self) -> Option<String> {
            self.0.get_one::<String>(Self::CITY_NAME).map_or(None, |arg| Some(arg.to_string()))
        }
        pub fn state_id(&self) -> Option<String> {
            self.0.get_one::<String>(Self::STATE_ID).map_or(None, |arg| Some(arg.to_string()))
        }
        pub fn state(&self) -> Option<String> {
            self.0.get_one::<String>(Self::STATE).map_or(None, |arg| Some(arg.to_string()))
        }
        pub fn latitude(&self) -> Option<String> {
            self.0.get_one::<String>(Self::LATITUDE).map_or(None, |arg| Some(arg.to_string()))
        }
        pub fn longitude(&self) -> Option<String> {
            self.0.get_one::<String>(Self::LONGITUDE).map_or(None, |arg| Some(arg.to_string()))
        }
        pub fn tz(&self) -> Option<String> {
            self.0.get_one::<String>(Self::TZ).map_or(None, |arg| Some(arg.to_string()))
        }
    }
    impl<'a> From<&'a ArgMatches> for LocationArgs<'a> {
        fn from(args: &'a ArgMatches) -> Self {
            Self(args)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn args() {
            fn parse(cmd: &mut Command, args: &[&str]) -> ArgMatches {
                let mut raw_args = cmd.try_get_matches_from_mut(args).unwrap();
                let (_, args) = raw_args.remove_subcommand().unwrap();
                args
            }
            let mut cmd = Command::new("test")
                .no_binary_name(true)
                .subcommand(Command::new("testcase").args(LocationArgs::get(false)));
            let cmd_args = parse(&mut cmd, &["testcase"]);
            let testcase = LocationArgs(&cmd_args);
            assert!(testcase.is_none());
            assert!(testcase.city().is_none());
            assert!(testcase.state_id().is_none());
            assert!(testcase.state().is_none());
            assert!(testcase.latitude().is_none());
            assert!(testcase.longitude().is_none());
            assert!(testcase.tz().is_none());

            macro_rules! value_arg {
                ($arg:expr, $value:expr) => {
                    ($value.to_string(), format!("--{}={}", $arg, $value))
                };
            }
            let (city, city_arg) = value_arg!("city", "Some City");
            let (state_id, state_id_arg) = value_arg!("id", "ST");
            let (state, state_arg) = value_arg!("state", "State");
            let (latitude, latitude_arg) = value_arg!("lat", "12.345");
            let (longitude, longitude_arg) = value_arg!("lon", "54.321");
            let (tz, tz_arg) = value_arg!("tz", "utc");
            let args = ["testcase", &city_arg, &state_id_arg, &state_arg, &latitude_arg, &longitude_arg, &tz_arg];
            let mut cmd = Command::new("test")
                .no_binary_name(true)
                .subcommand(Command::new("testcase").args(LocationArgs::get(true)));
            let cmd_args = parse(&mut cmd, &args);
            let testcase = LocationArgs(&cmd_args);
            assert!(!testcase.is_none());
            assert_eq!(testcase.city().unwrap(), city);
            assert_eq!(testcase.state_id().unwrap(), state_id);
            assert_eq!(testcase.state().unwrap(), state);
            assert_eq!(testcase.latitude().unwrap(), latitude);
            assert_eq!(testcase.longitude().unwrap(), longitude);
            assert_eq!(testcase.tz().unwrap(), tz);
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
            let month_opt = match name.to_lowercase().as_str() {
                "jan" => Some(1),
                "feb" => Some(2),
                "mar" => Some(3),
                "apr" => Some(4),
                "may" => Some(5),
                "jun" => Some(6),
                "jul" => Some(7),
                "aug" => Some(8),
                "sep" => Some(9),
                "oct" => Some(10),
                "nov" => Some(11),
                "dec" => Some(12),
                _ => {
                    parsed_date.replace(Err(format!("'{name}' is not a valid month name.")));
                    None
                }
            };
            if let Some(month) = month_opt {
                let year = cap["y"].parse::<i32>().unwrap();
                parsed_date.replace(Ok(NaiveDate::from_ymd_opt(year, month, 1).unwrap()));
            }
        }
        parsed_date
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

    /// Try to parse the string as a numeric day, month and year argument.
    ///
    /// # Arguments
    ///
    /// * `date_arg` is the string that will be parsed.
    ///
    fn try_full_date(date: &str) -> Option<Result<NaiveDate, String>> {
        let mut captures = MONTH_DAY_YEAR_RE.captures(date);
        if captures.is_none() {
            captures = YEAR_MONTH_DAY_RE.captures(date);
        }
        let mut parsed_date = None;
        if let Some(cap) = captures {
            let month = cap["m"].parse::<u32>().unwrap();
            if month < 1 || month > 12 {
                parsed_date.replace(Err("The dates month is out of bounds.".to_string()));
            } else {
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
        #[test]
        fn parse_daterange() {
            let testcase = try_parse_daterange("2025", None).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
            let testcase = try_parse_daterange("2025", Some("2025")).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
            let testcase = try_parse_daterange("mar-2025", None).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 3, 31).unwrap());
            let testcase = try_parse_daterange("mar-2025", Some("mar-2025")).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 3, 31).unwrap());
            let testcase = try_parse_daterange("03-01-2025", None).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
            let testcase = try_parse_daterange("2024", Some("2025")).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
            let testcase = try_parse_daterange("oct-2025", Some("nov-2025")).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 11, 30).unwrap());
            let testcase = try_parse_daterange("05-01-2025", None).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2025, 5, 1).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 5, 1).unwrap());
            let testcase = try_parse_daterange("06-02-2025", Some("06-15-2025")).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2025, 6, 2).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
            let testcase = try_parse_daterange("03-2025", Some("2025")).unwrap();
            assert_eq!(testcase.start, NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
            assert_eq!(testcase.end, NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
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
            let testcase = to_eom(&NaiveDate::from_ymd_opt(2023, 2, 14).unwrap());
            assert_eq!(testcase.year(), 2023);
            assert_eq!(testcase.month(), 2);
            assert_eq!(testcase.day(), 28);
            let testcase = to_eom(&NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
            assert_eq!(testcase.year(), 2024);
            assert_eq!(testcase.month(), 2);
            assert_eq!(testcase.day(), 29);
        }
        #[test]
        fn eoy() {
            let testcase = to_eoy(&NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
            assert_eq!(testcase.year(), 1970);
            assert_eq!(testcase.month(), 12);
            assert_eq!(testcase.day(), 31);
        }
        #[test]
        fn parse() {
            assert_eq!(try_parse("2025").unwrap(), NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
            assert_eq!(try_parse("02-2025").unwrap(), NaiveDate::from_ymd_opt(2025, 2, 1).unwrap());
            assert_eq!(try_parse("Mar-2025").unwrap(), NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
            assert_eq!(try_parse("04-15-2025").unwrap(), NaiveDate::from_ymd_opt(2025, 4, 15).unwrap());
            assert!(try_parse("June-2025").is_err());
        }
        #[test]
        fn yyyy() {
            assert!(try_year("1").is_none());
            assert!(try_year("19").is_none());
            assert!(try_year("197").is_none());
            assert_eq!(try_year("1970"), NaiveDate::from_ymd_opt(1970, 1, 1));
            assert!(try_year("19700").is_none());
        }
        #[test]
        fn text_month_year() {
            assert!(try_text_month_year("123-1970").is_none());
            assert!(try_text_month_year("J-1970").is_none());
            assert!(try_text_month_year("JA-1970").is_none());
            let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
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
            let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
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
            let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
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
        }
    }
}
