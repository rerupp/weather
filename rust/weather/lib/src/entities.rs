//! Structures used by the weather data `API`s.

use chrono::{NaiveDate, NaiveDateTime};

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
#[derive(Clone, Debug)]
pub struct Location {
    /// The name of the city.
    pub city: String,
    /// The short state name.
    pub state_id: String,
    /// The full state name.
    pub state: String,
    /// The name of a location.
    pub name: String,
    /// A unique nickname of a location.
    pub alias: String,
    /// The location latitude.
    pub latitude: String,
    /// The location longitude.
    pub longitude: String,
    /// the location timezone.
    pub tz: String,
}

/// The data that identifies selection of a location or locations.
#[derive(Debug)]
pub struct LocationFilter {
    /// A location can be searched by the city name.
    pub city: Option<String>,

    /// A location can be searched by the state name (full or two-letter form).
    pub state: Option<String>,

    /// A location can be searched for by its name or alias.
    pub name: Option<String>,
}
impl Default for LocationFilter {
    fn default() -> Self {
        Self { city: None, state: None, name: None }
    }
}
impl LocationFilter {
    /// A builder method that adds a city name to the filter.
    ///
    /// # Arguments
    ///
    /// * `city` is the name of the city.
    ///
    pub fn with_city(mut self, city: &str) -> Self {
        self.city.replace(String::from(city));
        self
    }

    /// A builder method that adds a state name to the filter.
    ///
    /// # Arguments
    ///
    /// * `state` is the name of the state.
    ///
    pub fn with_state(mut self, state: &str) -> Self {
        self.state.replace(String::from(state));
        self
    }

    /// A builder method that adds a location name to the filter.
    ///
    /// # Arguments
    ///
    /// * `name` is the name of the location.
    ///
    pub fn with_name(mut self, name: &str) -> Self {
        self.name.replace(String::from(name));
        self
    }

    /// Returns true if the city, state, and name are NONE.
    ///
    pub fn is_none(&self) -> bool {
        self.city.is_none() && self.state.is_none() && self.name.is_none()
    }
}

/// The location filter macro provides a simple front end to the [LocationFilter] builder.
///
#[macro_export]
macro_rules! location_filter {
    (city=$city:expr, state=$state:expr) => {
        $crate::prelude::LocationFilter::default().with_city($city).with_state($state)
    };
    (city=$city:expr) => {
        $crate::prelude::LocationFilter::default().with_city($city)
    };
    (state=$state:expr) => {
        $crate::prelude::LocationFilter::default().with_state($state)
    };
    (name=$name:expr) => {
        $crate::prelude::LocationFilter::default().with_name($name)
    };
    () => {
        $crate::prelude::LocationFilter::default()
    };
}

/// The collection of location filters. Originally this was defined as a type but having
/// a concrete class helps a bit with the Python library.
///
pub struct LocationFilters(
    /// The collection of location filters.
    Vec<LocationFilter>,
);
impl Default for LocationFilters {
    /// The default will have an empty collection of filters.
    fn default() -> Self {
        Self(vec![])
    }
}
impl IntoIterator for LocationFilters {
    type Item = LocationFilter;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    /// Return the collection of filters as an iterator.
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl LocationFilters {
    /// Create a new instance of the filters.
    ///
    /// # Arguments
    ///
    /// * `filters` is the collection of location filters.
    ///
    pub fn new(filters: Vec<LocationFilter>) -> Self {
        Self(filters)
    }

    /// This will return true if there are no filters available.
    ///
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return an iterator over the filter collection.
    ///
    pub fn iter(&self) -> std::slice::Iter<LocationFilter> {
        self.0.iter()
    }

    /// Return a mutable iterator over the filter collection.
    ///
    pub fn iter_mut(&mut self) -> std::slice::IterMut<LocationFilter> {
        self.0.iter_mut()
    }
}

/// The location filters macro provides a front-end to creating a location filters instance.
///
#[macro_export]
macro_rules! location_filters {
    () => {
        $crate::prelude::LocationFilters::default()
    };
    // lets this macro act like the vec! macro
    ($($x:expr),+ $(,)?) => {
        $crate::prelude::LocationFilters::new(vec![$($x),+])
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
        dates.sort_unstable();
        let mut ranges = vec![];
        let dates_len = dates.len();
        if dates_len == 1 {
            ranges.push(DateRange::new(dates[0], dates[0]));
        } else if dates_len > 1 {
            let mut from = dates[0];
            let mut to = dates[0];
            for i in 1..dates_len {
                if next_day!(to) != dates[i] {
                    ranges.push(DateRange::new(from, to));
                    from = dates[i];
                    to = dates[i];
                } else {
                    to = dates[i];
                }
            }
            ranges.push(DateRange::new(from, to));
        }
        Self { location_id: location_id.to_string(), date_ranges: ranges }
    }
    pub fn covers(&self, date: &NaiveDate) -> bool {
        self.date_ranges.iter().any(|date_range| date_range.covers(date))
    }
}

/// A container for a range of dates.
#[derive(Debug, PartialEq)]
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
    /// * `from` is the starting date.
    /// * `thru` is the inclusive end date.
    pub fn new(start: NaiveDate, end: NaiveDate) -> DateRange {
        DateRange { start, end }
    }
    /// Returns `true` if the *from* and *to* dates are equal.
    pub fn is_one_day(&self) -> bool {
        &self.start == &self.end
    }
    /// Identifies if a date is within the date range.
    ///
    /// # Arguments
    ///
    /// * `date` is the date that will be checked.
    pub fn covers(&self, date: &NaiveDate) -> bool {
        date >= &self.start && date <= &self.end
    }
    /// Allow the history range to be iterated over without consuming it.
    pub fn iter(&self) -> DateRangeIterator {
        DateRangeIterator { from: self.start, thru: self.end }
    }
    /// Returns the dates as a tuple of ISO8601 formatted strings.
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
    pub state: Option<String>,

    /// The optional zip code.
    pub zip_code: Option<String>,

    /// Limits the number of matches that will be returned.
    pub limit: usize,
}
/// The default limit is set at 25.
impl Default for CityFilter {
    fn default() -> Self {
        Self { name: None, state: None, zip_code: None, limit: 25 }
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
    pub fn iterate() {
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
    fn is_within() {
        let testcase = DateRange::new(get_date(2023, 7, 1), get_date(2023, 7, 31));
        assert!(testcase.covers(&get_date(2023, 7, 1)));
        assert!(!testcase.covers(&get_date(2023, 6, 30)));
        assert!(testcase.covers(&get_date(2023, 7, 31)));
        assert!(!testcase.covers(&get_date(2023, 8, 1)));
    }

    #[test]
    pub fn to_iso8601_history_range() {
        let test_case = DateRange::new(get_date(2022, 7, 1), get_date(2022, 7, 2));
        let (from, to) = test_case.as_iso8601();
        assert_eq!(from, "2022-07-01");
        assert_eq!(to, "2022-07-02");
    }

    #[test]
    pub fn date_ranges() {
        let testcase = DateRanges::new(
            "test",
            vec![
                get_date(2025, 5, 1),
                get_date(2025, 5, 3),
                get_date(2025, 5, 4),
                get_date(2025, 5, 6),
                get_date(2025, 5, 8),
                get_date(2025, 5, 9),
            ],
        );
        assert_eq!(testcase.location_id, "test");
        assert_eq!(testcase.date_ranges.len(), 4);
        assert_eq!(testcase.date_ranges[0], DateRange::new(get_date(2025, 5, 1), get_date(2025, 5, 1)));
        assert_eq!(testcase.date_ranges[1], DateRange::new(get_date(2025, 5, 3), get_date(2025, 5, 4)));
        assert_eq!(testcase.date_ranges[2], DateRange::new(get_date(2025, 5, 6), get_date(2025, 5, 6)));
        assert_eq!(testcase.date_ranges[3], DateRange::new(get_date(2025, 5, 8), get_date(2025, 5, 9)));
    }

    // #[test]
    // pub fn location_criteria() {
    //     let default = LocationCriteria::default();
    //     assert!(default.filter.is_none());
    //     assert!(default.include_all());
    //     assert_eq!(default.limit, usize::MAX);
    //
    //     let empty = LocationCriteria::new(None, None, None, None);
    //     assert!(empty.filter.is_none());
    //     assert!(empty.include_all());
    //     assert_eq!(empty.limit, usize::MAX);
    //
    //     let city = "City".to_string();
    //     let state = "State".to_string();
    //     let name = "Name".to_string();
    //     let full = LocationCriteria::new(Some(city.clone()), Some(state.clone()), Some(name.clone()), Some(250));
    //     assert!(!full.include_all());
    //     assert_eq!(full.filter.city, Some(city));
    //     assert_eq!(full.filter.state, Some(state));
    //     assert_eq!(full.filter.name, Some(name));
    //     assert_eq!(full.limit, 250);
    // }

    #[test]
    pub fn location_filter() {
        let testcase = LocationFilter::default();
        assert!(testcase.is_none());

        let testcase = LocationFilter::default().with_city("city");
        assert!(!testcase.is_none());
        assert_eq!(testcase.city.unwrap(), "city");
        assert!(testcase.state.is_none());
        assert!(testcase.name.is_none());

        let testcase = LocationFilter::default().with_state("state");
        assert!(!testcase.is_none());
        assert!(testcase.city.is_none());
        assert_eq!(testcase.state.unwrap(), "state");
        assert!(testcase.name.is_none());

        let testcase = LocationFilter::default().with_name("name");
        assert!(!testcase.is_none());
        assert!(testcase.city.is_none());
        assert!(testcase.state.is_none());
        assert_eq!(testcase.name.unwrap(), "name");
    }

    #[test]
    fn location_filter_macro() {
        let testcase = location_filter!();
        assert!(testcase.is_none());

        let testcase = location_filter!(city = "City");
        assert!(!testcase.is_none());
        assert_eq!(testcase.city.unwrap(), "City");
        assert!(testcase.state.is_none());
        assert!(testcase.name.is_none());

        let testcase = location_filter!(state = "State");
        assert!(!testcase.is_none());
        assert!(testcase.city.is_none());
        assert_eq!(testcase.state.unwrap(), "State");
        assert!(testcase.name.is_none());

        let testcase = location_filter!(name = "Name");
        assert!(!testcase.is_none());
        assert!(testcase.city.is_none());
        assert!(testcase.state.is_none());
        assert_eq!(testcase.name.unwrap(), "Name");
    }
}
