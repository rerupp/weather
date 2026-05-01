//! Structures used by the weather data API.

use chrono::{Datelike, Days, NaiveDate, NaiveDateTime};

/// A locations daily weather history.
#[derive(Debug)]
pub struct DailyHistories {
    /// The location metadata.
    pub location: Location,
    /// The daily histories for a location.
    pub histories: Vec<History>,
}

/// A locations history dates.
#[derive(Debug)]
pub struct HistoryDates {
    /// The location metadata.
    pub location: Location,
    /// The history dates metadata.
    pub history_dates: Vec<DateRange>,
}

#[derive(Debug)]
/// A locations history summary.
pub struct HistorySummaries {
    pub location: Location,
    /// The number of weather data histories available.
    pub count: usize,
    /// The overall size of weather data in bytes (may or may not be available).
    pub overall_size: Option<usize>,
    /// The size in bytes of weather data.
    pub raw_size: Option<usize>,
    /// The size in bytes of weather data in the backing store.
    pub store_size: Option<usize>,
}

/// The data that comprises a location.
#[derive(Clone, Debug, Default)]
pub struct Location {
    /// The country name such as *United States* or *Canada*.
    pub country_name: String,
    /// The country code such as *US* or *CA*.
    pub country_code: String,
    /// The region name such as *Arizona* or *British Columbia*.
    pub region_name: String,
    /// The region code such as *AZ* or *BC*.
    pub region_code: String,
    /// The name of the city.
    pub city_name: String,
    /// A unique nickname of a location.
    pub alias: String,
    /// The location latitude.
    pub latitude: String,
    /// The location longitude.
    pub longitude: String,
    /// the location timezone.
    pub tz: String,
}
impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {} ({})", self.city_name, self.region_code, self.alias)
    }
}
/// This macro will order locations by comparing city_name, region_code, country_code, and alias.
macro_rules! location_order {
    ($lhs: expr, $rhs: expr) => {
        match $lhs.city_name.cmp(&$rhs.city_name) {
            std::cmp::Ordering::Equal => match $lhs.region_code.cmp(&$rhs.region_code) {
                std::cmp::Ordering::Equal => match $lhs.country_code.cmp(&$rhs.country_code) {
                    std::cmp::Ordering::Equal => $lhs.alias.cmp(&$rhs.alias),
                    country_code_ordering => country_code_ordering,
                },
                region_code_ordering => region_code_ordering,
            },
            city_name_ordering => city_name_ordering,
        }
    };
}
pub(crate) use location_order;
impl Ord for Location {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        location_order!(self, other)
    }
}
impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
/// This macro is used to check if two locations match by checking the alias, city_name,
/// region_code, and country_code.
///
macro_rules! location_equal {
    ($lhs: expr, $rhs: expr) => {
        $lhs.alias == $rhs.alias
            && $lhs.city_name == $rhs.city_name
            && $lhs.region_code == $rhs.region_code
            && $lhs.country_code == $rhs.country_code
    };
}
pub(crate) use location_equal;
impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        location_equal!(self, other)
    }
}
impl Eq for Location {}

/// The data that identifies selection of a location or locations.
///
#[derive(Clone, Debug, Default)]
pub struct LocationFilter {
    /// Locations can be searched by the alias name.
    pub alias: Option<String>,
    /// Locations can be searched by city name.
    pub city: Option<String>,
    /// Locations can be searched by the region name or code.
    pub region: Option<String>,
    /// Locations can be searched for by the country name or code.
    pub country: Option<String>,
}
impl std::fmt::Display for LocationFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LocationFilter {{")?;
        if let Some(alias) = &self.alias {
            write!(f, " alias=\"{alias}\"")?;
        }
        if let Some(city) = &self.city {
            write!(f, " city=\"{}\"", city)?;
        }
        if let Some(region) = &self.region {
            write!(f, " region=\"{}\"", region)?;
        }
        if let Some(country) = &self.country {
            write!(f, " country=\"{}\"", country)?;
        }
        write!(f, " }}")
    }
}
impl LocationFilter {
    /// Create a new location filter initialized with an alias name.
    ///
    /// # Arguments
    ///
    /// * `alias` is the location alias name.
    ///
    pub fn alias(alias: impl Into<String>) -> LocationFilter {
        Self { alias: Some(alias.into()), ..Default::default() }
    }

    /// Create a new location filter initialized with a city name.
    ///
    /// # Arguments
    ///
    /// * `city` is the name of the city.
    ///
    pub fn city(city: impl Into<String>) -> LocationFilter {
        Self { city: Some(city.into()), ..Default::default() }
    }

    /// Create a new location filter initialized with a region name or code.
    ///
    /// # Arguments
    ///
    /// * `region` is the region name or code.
    ///
    pub fn region(region: impl Into<String>) -> LocationFilter {
        Self { region: Some(region.into()), ..Default::default() }
    }

    /// Create a new location filter initialized with a country name or code.
    ///
    /// # Arguments
    ///
    /// * `country` is the country name or code.
    ///
    pub fn country(country: impl Into<String>) -> LocationFilter {
        Self { country: Some(country.into()), ..Default::default() }
    }

    /// A builder method that adds a city name to the filter.
    ///
    /// # Arguments
    ///
    /// * `city` is the name of the city.
    ///
    pub fn with_city(mut self, city: impl Into<String>) -> Self {
        self.city.replace(city.into());
        self
    }

    /// A builder method that adds a region name or code to the filter.
    ///
    /// # Arguments
    ///
    /// * `region` is the name of the state.
    ///
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region.replace(region.into());
        self
    }

    /// A builder method that adds a country name or code to the filter.
    ///
    /// # Arguments
    ///
    /// * `country` is the name of the location.
    ///
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country.replace(country.into());
        self
    }

    /// Returns true if the city, state, and name are NONE.
    ///
    pub fn is_none(&self) -> bool {
        self.alias.is_none() && self.city.is_none() && self.region.is_none() && self.country.is_none()
    }
}

/// A locations history summary.
#[derive(Debug)]
pub struct HistorySummary {
    /// The location id.
    pub location_id: String,
    /// The number of weather data histories available.
    pub count: usize,
    /// The overall size of weather data for a location in bytes (may or may not be available).
    pub overall_size: Option<usize>,
    /// The raw size of weather data for a location in bytes (may or may not be available).
    pub raw_size: Option<usize>,
    /// The compressed data size of weather data for a location in bytes (may or may not be available).
    pub compressed_size: Option<usize>,
}

/// The weather history data.
#[derive(Debug, Default)]
pub struct History {
    /// The location alias name.
    pub alias: String,
    /// The history date.
    pub date: NaiveDate,
    /// The high temperature for the day.
    pub temperature_high: Option<f64>,
    /// The low temperature for the day.
    pub temperature_low: Option<f64>,
    /// The daily mean temperature.
    pub temperature_mean: Option<f64>,
    /// The dew point temperature.
    pub dew_point: Option<f64>,
    /// The relative humidity percentage.
    pub humidity: Option<f64>,
    /// The chance of rain during the day.
    pub precipitation_chance: Option<f64>,
    /// A short description of the type of rain.
    pub precipitation_type: Option<String>,
    /// The amount of precipitation for the day.
    pub precipitation_amount: Option<f64>,
    /// The daily wind speed.
    pub wind_speed: Option<f64>,
    /// The highest wind speed recorded for the day.
    pub wind_gust: Option<f64>,
    /// The general direction in degrees.
    pub wind_direction: Option<i64>,
    /// The percentage of sky covered by clouds.
    pub cloud_cover: Option<f64>,
    /// The daily atmospheric pressure expressed in millibars.
    pub pressure: Option<f64>,
    /// The level of ultraviolet exposure for the day.
    pub uv_index: Option<f64>,
    /// The local time when the sun comes up.
    pub sunrise: Option<NaiveDateTime>,
    /// The local time when the sun will set.
    pub sunset: Option<NaiveDateTime>,
    /// The moons phase between 0 and 1.
    pub moon_phase: Option<f64>,
    /// The distance that can be during the day.
    pub visibility: Option<f64>,
    /// A summary of the daily weather.
    pub description: Option<String>,
}
impl History {
    /// A type-method that accepts a compass bearing and returns a human readable direction.
    ///
    /// The four cardinal points on a compass are subdivided into a finer grained
    /// direction strings as shown below:
    /// ```text
    /// N NNE NE ENE
    /// E ESE SE SSE
    /// S SSW SW WSW
    /// W WNW NW NNW
    /// ```
    ///
    /// There is a window around the absolute direction to determine the bearing string.
    /// As an example any bearing between 348.75 degrees and 11.25 degrees will be returned
    /// as a `N` bearing string.
    ///
    /// If the option is `None` an empty string will be returned.
    ///
    /// # Arguments
    ///
    /// * `wind_direction` - the bearing that will be converter to a string.
    ///
    pub fn wind_direction_str(wind_direction: Option<i64>) -> &'static str {
        static BEARINGS: [&'static str; 16] =
            ["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"];
        match wind_direction {
            Some(direction) => BEARINGS[(direction as f64 / 22.5).round() as usize % 16],
            _ => Default::default(),
        }
    }

    /// A type-method that accepts a UV index and returns a human readable string.
    ///
    /// The possible UV index strings are:
    ///
    /// | UV Index | Description |
    /// | :----: | :----: |
    /// | 1-2 | low |
    /// | 3-5 | moderate |
    /// | 6-7 | high |
    /// | 8-10 | very high |
    /// | 11+ | extreme |
    ///
    /// If the option is `None` or the value 0, an empty string will be returned.
    ///
    pub fn uv_index_str(uv_index: Option<f64>) -> &'static str {
        match uv_index {
            Some(uv_index) if uv_index > 0.0 => match uv_index.round() as usize {
                1 | 2 => "low",
                3 | 4 | 5 => "moderate",
                6 | 7 => "high",
                8 | 9 | 10 => "very high",
                _ => "extreme",
            },
            _ => Default::default(),
        }
    }

    /// A type-method that accepts a moon phase and returns a human readable string.
    ///
    /// The possible moon phase indicators are:
    ///
    /// | Moon Phase | Description |
    /// | :----: | :----: |
    /// | 0 | new moon |
    /// | 0-0.25 | waxing crescent |
    /// | 0.25 | first quarter |
    /// | 0.25-0.5 | waxing gibbous |
    /// | 0.5 | full moon |
    /// | 0.5-0.75 | waning gibbous |
    /// | 0.75 | last quarter |
    /// | 0.75-1.0 | waning crescent |
    ///
    /// If the option is `None` an empty string will be returned.
    ///
    pub fn moon_phase_str(moon_phase: Option<f64>) -> &'static str {
        match moon_phase {
            Some(moon_phase) => {
                if moon_phase >= 0.0 && moon_phase <= 0.01 {
                    "new moon"
                } else if moon_phase > 0.01 && moon_phase < 0.24 {
                    "waxing crescent"
                } else if moon_phase >= 0.24 && moon_phase <= 0.26 {
                    "first quarter"
                } else if moon_phase > 0.26 && moon_phase < 0.49 {
                    "waxing gibbous"
                } else if moon_phase >= 0.49 && moon_phase <= 0.51 {
                    "full moon"
                } else if moon_phase > 0.51 && moon_phase < 0.74 {
                    "waning gibbous"
                } else if moon_phase >= 0.74 && moon_phase <= 0.76 {
                    "last quarter"
                } else if moon_phase > 0.76 && moon_phase <= 1.0 {
                    "waning crescent"
                } else {
                    "unknown"
                }
            }
            _ => Default::default(),
        }
    }
}

/// For a given `NaiveDate` return the next day `NaiveDate`.
macro_rules! next_day {
    ($nd:expr) => {
        // For the weather data use case this should always be okay
        $nd.succ_opt().unwrap()
    };
}

/// A locations weather data history dates.
#[derive(Debug)]
pub struct DateRanges {
    /// The location id.
    pub location_id: String,
    /// The location weather history dates, grouped as consecutive date ranges.
    pub date_ranges: Vec<DateRange>,
}
impl DateRanges {
    pub fn new(location_id: &str, mut dates: Vec<NaiveDate>) -> Self {
        match dates.is_empty() {
            true => Self { location_id: location_id.to_string(), date_ranges: vec![] },
            false => {
                dates.sort_unstable();
                // walk the dates collection and create all the date ranges
                let mut date_ranges: Vec<DateRange> = vec![];
                let mut start = dates[0];
                let mut end = start;
                for date in dates.drain(1..) {
                    // capture date ranges on a years boundary
                    if date.year() != end.year() {
                        date_ranges.push(DateRange::new(start, end));
                        start = date;
                        end = date;
                    } else if next_day!(end) != date {
                        date_ranges.push(DateRange::new(start, end));
                        start = date;
                        end = date;
                    } else {
                        end = date;
                    }
                }
                date_ranges.push(DateRange::new(start, end));
                // now walk back through the date ranges collecting consecutive yearly date ranges
                let mut self_ = Self { location_id: location_id.to_string(), date_ranges: vec![] };
                for date_range in date_ranges {
                    match self_.date_ranges.last_mut() {
                        None => self_.date_ranges.push(date_range),
                        Some(last_date_range) => match date_range.is_one_year() {
                            false => self_.date_ranges.push(date_range),
                            true => match last_date_range.is_one_year() || last_date_range.is_multi_year() {
                                false => self_.date_ranges.push(date_range),
                                true => match next_day!(last_date_range.end) == date_range.start {
                                    false => self_.date_ranges.push(date_range),
                                    true => last_date_range.end = date_range.end,
                                },
                            },
                        },
                    }
                }
                self_
            }
        }
    }
    pub fn covers(&self, date: &NaiveDate) -> bool {
        self.date_ranges.iter().any(|date_range| date_range.contains(date))
    }
}

/// A container for a range of dates.
#[derive(Clone, Debug, PartialEq)]
pub struct DateRange {
    /// The starting date of the range.
    pub start: NaiveDate,
    /// The inclusive end date of the range.
    pub end: NaiveDate,
}
impl DateRange {
    /// Create a new instance of the date range.
    ///
    /// # Arguments
    ///
    /// * `start` is the starting date.
    /// * `end` is the inclusive end date.
    ///
    pub fn new(start: NaiveDate, end: NaiveDate) -> DateRange {
        DateRange { start, end }
    }

    /// Returns `true` if the *from* and *to* dates are equal.
    ///
    pub fn is_one_day(&self) -> bool {
        &self.start == &self.end
    }

    /// Returns `true` if the date range covers an entire year.
    ///
    pub fn is_one_year(&self) -> bool {
        if self.start.month() == 1 && self.start.day() == 1 {
            if self.end.month() == 12 && self.end.day() == 31 {
                if self.start.year() == self.end.year() {
                    return true;
                }
            }
        }
        false
    }

    /// Returns `true` if the date range covers multiple entire years.
    ///
    pub fn is_multi_year(&self) -> bool {
        if self.start.month() == 1 && self.start.day() == 1 {
            if self.end.month() == 12 && self.end.day() == 31 {
                if self.start.year() != self.end.year() {
                    return true;
                }
            }
        }
        false
    }

    /// Convert the date range into an annualized collection of date ranges.
    ///
    pub fn annualized(&self) -> Vec<DateRange> {
        let mut date_ranges = vec![];
        // check if the date range is within the same year
        if self.start.year() == self.end.year() {
            date_ranges.push(DateRange::new(self.start, self.end));
        } else {
            // there are at least 2 annualized years at this point
            macro_rules! eoy {
                ($year:expr) => {
                    NaiveDate::from_ymd_opt($year, 12, 31).unwrap()
                };
            }

            // add the first annualized date range
            let mut year = self.start.year();
            date_ranges.push(DateRange::new(self.start, eoy!(year)));
            year += 1;

            // now walk the years until you're outside the date range
            while let Some(start_date) = NaiveDate::from_ymd_opt(year, 1, 1) {
                // you're done once the start date is outside the date range
                if !self.contains(&start_date) {
                    break;
                }

                // you're done if the last day of the year is outside the date range
                let end_of_year = eoy!(year);
                if !self.contains(&end_of_year) {
                    date_ranges.push(DateRange::new(start_date, self.end));
                    break;
                }

                // save the date range and move onto the next year
                date_ranges.push(DateRange::new(start_date, end_of_year));
                year += 1;
            }
        }
        date_ranges
    }

    /// Identifies if a date is greater than or equal to the start date and less than or
    /// equal to the end date.
    ///
    /// # Arguments
    ///
    /// * `date` is the date that will be checked.
    ///
    pub fn contains(&self, date: &NaiveDate) -> bool {
        date >= &self.start && date <= &self.end
    }

    /// Allow the history range to be iterated over without consuming it.
    ///
    pub fn iter(&self) -> DateRangeIterator {
        DateRangeIterator { from: self.start, thru: self.end }
    }

    /// Returns the dates as a tuple of ISO8601 formatted strings.
    ///
    pub fn as_iso8601(&self) -> (String, String) {
        use toolslib::date_time::isodate;
        (isodate(&self.start), isodate(&self.end))
    }
}
/// Create an iterator that will return all dates within the range.
impl IntoIterator for DateRange {
    type Item = NaiveDate;
    type IntoIter = DateRangeIterator;
    fn into_iter(self) -> Self::IntoIter {
        DateRangeIterator { from: self.start, thru: self.end }
    }
}
/// Create an iterator that will return all dates within the range.
impl IntoIterator for &DateRange {
    type Item = NaiveDate;
    type IntoIter = DateRangeIterator;
    fn into_iter(self) -> Self::IntoIter {
        DateRangeIterator { from: self.start, thru: self.end }
    }
}
impl std::fmt::Display for DateRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DAY_FMT: &'static str = "%b-%d-%Y";
        const MONTH_FMT: &'static str = "%b-%Y";
        if self.is_one_day() {
            return write!(f, "{}", self.start.format(DAY_FMT));
        }
        if self.is_one_year() {
            return write!(f, "{:04}", self.start.year());
        }
        if self.is_multi_year() {
            return write!(f, "{:04} thru {:04}", self.start.year(), self.end.year());
        }
        let is_at_som = self.start.day() == 1;
        let is_at_eom = self.end.checked_add_days(Days::new(1)).unwrap().day() == 1;
        if is_at_som && is_at_eom {
            return if self.start.year() == self.end.year() && self.start.month() == self.end.month() {
                write!(f, "{}", self.start.format(MONTH_FMT))
            } else {
                write!(f, "{} thru {}", self.start.format(MONTH_FMT), self.end.format(MONTH_FMT))
            };
        }
        let is_different_year_or_month = self.start.year() != self.end.year() || self.start.month() != self.end.month();
        if is_at_som {
            if is_different_year_or_month {
                return write!(f, "{} thru {}", self.start.format(MONTH_FMT), self.end.format(DAY_FMT));
            }
        } else if is_at_eom {
            if is_different_year_or_month {
                return write!(f, "{} thru {}", self.start.format(DAY_FMT), self.end.format(MONTH_FMT));
            }
        }
        write!(f, "{} thru {}", self.start.format(DAY_FMT), self.end.format(DAY_FMT))
    }
}

/// Create the DateRange iterator structure.
#[derive(Debug)]
///
/// # Arguments
///
/// * `from` is the starting date.
/// * `thru` is the inclusive end date.
pub struct DateRangeIterator {
    /// The starting date.
    from: NaiveDate,
    /// The inclusive end date.
    thru: NaiveDate,
}
/// The implementation of iterating over the date range.
impl Iterator for DateRangeIterator {
    type Item = NaiveDate;
    fn next(&mut self) -> Option<Self::Item> {
        if self.from > self.thru {
            None
        } else {
            let date = self.from;
            self.from = next_day!(date);
            Some(date)
        }
    }
}

/// The filter used to find cities.
#[derive(Debug)]
pub struct CityFilter {
    /// The optional city name.
    pub name: Option<String>,

    /// The optional state name.
    pub region: Option<String>,

    /// The optional zip code.
    pub country: Option<String>,

    /// Limits the number of matches that will be returned.
    pub limit: usize,
}
/// The default limit is set at 25.
impl Default for CityFilter {
    fn default() -> Self {
        Self { name: None, region: None, country: None, limit: 25 }
    }
}

/// The bean that holds city information.
#[derive(Debug)]
pub struct City {
    /// The city county name such as *United States* or *Canada*.
    pub country_name: String,
    /// The city country code such as *US* or *CA*.
    pub country_code: String,
    /// The city region name such as *Arizona* or *British Columbia*.
    pub region_name: String,
    /// The city region code such as *AZ* or *BC*.
    pub region_code: String,
    /// The name of the city.
    pub name: String,
    /// The city latitude.
    pub latitude: String,
    /// The city longitude.
    pub longitude: String,
    /// The city timezone.
    pub tz: String,
}
impl From<City> for Location {
    fn from(city: City) -> Self {
        Location {
            country_name: city.country_name,
            country_code: city.country_code,
            region_code: city.region_code,
            region_name: city.region_name,
            city_name: city.name,
            alias: String::default(),
            latitude: city.latitude,
            longitude: city.longitude,
            tz: city.tz,
        }
    }
}

/// The US City state names.
pub struct State {
    /// The states full name.
    pub name: String,

    /// The two letter state abbreviation.
    pub state_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use toolslib::date_time::get_date;

    #[test]
    pub fn date_range_iterate() {
        let range = DateRange::new(get_date(2022, 6, 1), get_date(2022, 6, 30));
        let mut testcase = range.start.clone();
        let test_cases: Vec<NaiveDate> = range.into_iter().collect();
        assert_eq!(test_cases.len(), 30);
        for day in 0..30 {
            assert_eq!(test_cases[day], testcase);
            // test_case = test_case.succ();
            testcase = next_day!(testcase);
        }
    }

    #[test]
    fn date_range_contains() {
        let testcase = DateRange::new(get_date(2023, 7, 1), get_date(2023, 7, 31));
        assert!(testcase.contains(&get_date(2023, 7, 1)));
        assert!(!testcase.contains(&get_date(2023, 6, 30)));
        assert!(testcase.contains(&get_date(2023, 7, 31)));
        assert!(!testcase.contains(&get_date(2023, 8, 1)));
    }

    #[test]
    pub fn date_range_to_iso8601_history_range() {
        let test_case = DateRange::new(get_date(2022, 7, 1), get_date(2022, 7, 2));
        let (from, to) = test_case.as_iso8601();
        assert_eq!(from, "2022-07-01");
        assert_eq!(to, "2022-07-02");
    }

    #[test]
    fn date_range_to_string() {
        macro_rules! date {
            ($y:expr, $m:expr, $d:expr) => {
                NaiveDate::from_ymd_opt($y, $m, $d).unwrap()
            };
        }
        macro_rules! testcase {
            ($start: expr, $end: expr) => {
                DateRange::new($start, $end).to_string()
            };
        }
        assert_eq!(testcase!(date!(2025, 11, 18), date!(2025, 11, 18)), "Nov-18-2025");
        assert_eq!(testcase!(date!(2025, 11, 2), date!(2025, 11, 30)), "Nov-02-2025 thru Nov-30-2025");
        assert_eq!(testcase!(date!(2025, 11, 1), date!(2025, 11, 29)), "Nov-01-2025 thru Nov-29-2025");
        assert_eq!(testcase!(date!(2025, 1, 1), date!(2025, 12, 31)), "2025");
        assert_eq!(testcase!(date!(2024, 1, 1), date!(2025, 12, 31)), "2024 thru 2025");
        assert_eq!(testcase!(date!(2025, 11, 1), date!(2025, 11, 30)), "Nov-2025");
        assert_eq!(testcase!(date!(2024, 11, 1), date!(2025, 11, 30)), "Nov-2024 thru Nov-2025");
        assert_eq!(testcase!(date!(2025, 11, 1), date!(2025, 12, 1)), "Nov-2025 thru Dec-01-2025");
        assert_eq!(testcase!(date!(2025, 11, 30), date!(2025, 12, 31)), "Nov-30-2025 thru Dec-2025");
        assert_eq!(testcase!(date!(2024, 11, 30), date!(2025, 12, 31)), "Nov-30-2024 thru Dec-2025");
    }

    #[test]
    fn daterange_annualized() {
        macro_rules! date {
            ($y:expr, $m:expr, $d:expr) => {
                NaiveDate::from_ymd_opt($y, $m, $d).unwrap()
            };
        }

        // check a partial year
        let testcase = DateRange::new(date!(2022, 1, 1), date!(2022, 12, 30)).annualized();
        assert_eq!(testcase.len(), 1);
        assert_eq!(&testcase[0], &DateRange::new(date!(2022, 1, 1), date!(2022, 12, 30)));

        // check partial years
        let testcase = DateRange::new(date!(2022, 12, 31), date!(2023, 1, 1)).annualized();
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase[0], DateRange::new(date!(2022, 12, 31), date!(2022, 12, 31)));
        assert_eq!(testcase[1], DateRange::new(date!(2023, 1, 1), date!(2023, 1, 1)));

        // check partial years
        let testcase = DateRange::new(date!(2023, 1, 1), date!(2025, 12, 31)).annualized();
        assert_eq!(testcase.len(), 3);
        assert_eq!(testcase[0], DateRange::new(date!(2023, 1, 1), date!(2023, 12, 31)));
        assert_eq!(testcase[1], DateRange::new(date!(2024, 1, 1), date!(2024, 12, 31)));
        assert_eq!(testcase[2], DateRange::new(date!(2025, 1, 1), date!(2025, 12, 31)));
    }

    #[test]
    pub fn date_ranges() {
        let mut dates = DateRange::new(get_date(2012, 1, 1), get_date(2012, 4, 30)).into_iter().collect::<Vec<_>>();
        dates.append(&mut DateRange::new(get_date(2019, 10, 1), get_date(2021, 4, 30)).into_iter().collect::<Vec<_>>());
        dates.append(&mut DateRange::new(get_date(2012, 6, 1), get_date(2012, 8, 31)).into_iter().collect::<Vec<_>>());
        dates.append(&mut DateRange::new(get_date(2018, 1, 1), get_date(2018, 12, 31)).into_iter().collect::<Vec<_>>());
        dates.append(&mut DateRange::new(get_date(2014, 1, 1), get_date(2016, 12, 31)).into_iter().collect::<Vec<_>>());
        let testcase = DateRanges::new("alias", dates);
        assert_eq!("alias", testcase.location_id);
        let expected = vec![
            DateRange::new(get_date(2012, 1, 1), get_date(2012, 4, 30)),
            DateRange::new(get_date(2012, 6, 1), get_date(2012, 8, 31)),
            DateRange::new(get_date(2014, 1, 1), get_date(2016, 12, 31)),
            DateRange::new(get_date(2018, 1, 1), get_date(2018, 12, 31)),
            DateRange::new(get_date(2019, 10, 1), get_date(2019, 12, 31)),
            DateRange::new(get_date(2020, 1, 1), get_date(2020, 12, 31)),
            DateRange::new(get_date(2021, 1, 1), get_date(2021, 4, 30)),
        ];
        assert_eq!(testcase.date_ranges.len(), expected.len());
        for (idx, date_range) in testcase.date_ranges.iter().enumerate() {
            assert_eq!(&expected[idx], date_range);
        }
    }

    #[test]
    pub fn location_filter() {
        let testcase = LocationFilter::default();
        assert!(testcase.is_none());

        let testcase = LocationFilter::default().with_city("city");
        assert!(!testcase.is_none());
        assert_eq!(testcase.city.unwrap(), "city");
        assert!(testcase.region.is_none());
        assert!(testcase.country.is_none());

        let testcase = LocationFilter::default().with_region("state");
        assert!(!testcase.is_none());
        assert!(testcase.city.is_none());
        assert_eq!(testcase.region.unwrap(), "state");
        assert!(testcase.country.is_none());

        let testcase = LocationFilter::default().with_country("name");
        assert!(!testcase.is_none());
        assert!(testcase.city.is_none());
        assert!(testcase.region.is_none());
        assert_eq!(testcase.country.unwrap(), "name");
    }

    #[test]
    fn moon_phase() {
        assert_eq!(History::moon_phase_str(None), "");
        assert_eq!(History::moon_phase_str(Some(0.0)), "new moon");
        assert_eq!(History::moon_phase_str(Some(0.01)), "new moon");
        assert_eq!(History::moon_phase_str(Some(0.011)), "waxing crescent");
        assert_eq!(History::moon_phase_str(Some(0.239)), "waxing crescent");
        assert_eq!(History::moon_phase_str(Some(0.24)), "first quarter");
        assert_eq!(History::moon_phase_str(Some(0.26)), "first quarter");
        assert_eq!(History::moon_phase_str(Some(0.261)), "waxing gibbous");
        assert_eq!(History::moon_phase_str(Some(0.489)), "waxing gibbous");
        assert_eq!(History::moon_phase_str(Some(0.49)), "full moon");
        assert_eq!(History::moon_phase_str(Some(0.51)), "full moon");
        assert_eq!(History::moon_phase_str(Some(0.511)), "waning gibbous");
        assert_eq!(History::moon_phase_str(Some(0.739)), "waning gibbous");
        assert_eq!(History::moon_phase_str(Some(0.74)), "last quarter");
        assert_eq!(History::moon_phase_str(Some(0.76)), "last quarter");
        assert_eq!(History::moon_phase_str(Some(0.761)), "waning crescent");
        assert_eq!(History::moon_phase_str(Some(1.0)), "waning crescent");
        assert_eq!(History::moon_phase_str(Some(1.001)), "unknown");
    }
    #[test]
    fn wind_bearing() {
        assert_eq!(History::wind_direction_str(None), "");
        assert_eq!(History::wind_direction_str(Some(0)), "N");
        assert_eq!(History::wind_direction_str(Some(11)), "N");
        assert_eq!(History::wind_direction_str(Some(12)), "NNE");
        assert_eq!(History::wind_direction_str(Some(33)), "NNE");
        assert_eq!(History::wind_direction_str(Some(34)), "NE");
        assert_eq!(History::wind_direction_str(Some(56)), "NE");
        assert_eq!(History::wind_direction_str(Some(57)), "ENE");
        assert_eq!(History::wind_direction_str(Some(78)), "ENE");
        assert_eq!(History::wind_direction_str(Some(79)), "E");
        assert_eq!(History::wind_direction_str(Some(101)), "E");
        assert_eq!(History::wind_direction_str(Some(102)), "ESE");
        assert_eq!(History::wind_direction_str(Some(123)), "ESE");
        assert_eq!(History::wind_direction_str(Some(124)), "SE");
        assert_eq!(History::wind_direction_str(Some(146)), "SE");
        assert_eq!(History::wind_direction_str(Some(147)), "SSE");
        assert_eq!(History::wind_direction_str(Some(168)), "SSE");
        assert_eq!(History::wind_direction_str(Some(169)), "S");
        assert_eq!(History::wind_direction_str(Some(191)), "S");
        assert_eq!(History::wind_direction_str(Some(192)), "SSW");
        assert_eq!(History::wind_direction_str(Some(213)), "SSW");
        assert_eq!(History::wind_direction_str(Some(214)), "SW");
        assert_eq!(History::wind_direction_str(Some(236)), "SW");
        assert_eq!(History::wind_direction_str(Some(237)), "WSW");
        assert_eq!(History::wind_direction_str(Some(258)), "WSW");
        assert_eq!(History::wind_direction_str(Some(259)), "W");
        assert_eq!(History::wind_direction_str(Some(281)), "W");
        assert_eq!(History::wind_direction_str(Some(282)), "WNW");
        assert_eq!(History::wind_direction_str(Some(303)), "WNW");
        assert_eq!(History::wind_direction_str(Some(304)), "NW");
        assert_eq!(History::wind_direction_str(Some(326)), "NW");
        assert_eq!(History::wind_direction_str(Some(327)), "NNW");
        assert_eq!(History::wind_direction_str(Some(348)), "NNW");
        assert_eq!(History::wind_direction_str(Some(349)), "N");
        assert_eq!(History::wind_direction_str(Some(360)), "N");
    }

    #[test]
    fn uv_index() {
        assert_eq!(History::uv_index_str(None), "");
        assert_eq!(History::uv_index_str(Some(0.0)), "");
        assert_eq!(History::uv_index_str(Some(1.0)), "low");
        assert_eq!(History::uv_index_str(Some(2.0)), "low");
        assert_eq!(History::uv_index_str(Some(3.0)), "moderate");
        assert_eq!(History::uv_index_str(Some(4.0)), "moderate");
        assert_eq!(History::uv_index_str(Some(5.0)), "moderate");
        assert_eq!(History::uv_index_str(Some(6.0)), "high");
        assert_eq!(History::uv_index_str(Some(7.0)), "high");
        assert_eq!(History::uv_index_str(Some(8.0)), "very high");
        assert_eq!(History::uv_index_str(Some(9.0)), "very high");
        assert_eq!(History::uv_index_str(Some(10.0)), "very high");
        assert_eq!(History::uv_index_str(Some(11.0)), "extreme");
        assert_eq!(History::uv_index_str(Some(12.0)), "extreme");
    }
}
