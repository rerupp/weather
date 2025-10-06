//! Structures used by the weather data `API`s.
use chrono::{Datelike, NaiveDate, NaiveDateTime};

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
                                true => match next_day!(last_date_range.end) == date_range.start{
                                    false => self_.date_ranges.push(date_range),
                                    true => last_date_range.end = date_range.end
                                }
                            }
                        }
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
    /// * `from` is the starting date.
    /// * `thru` is the inclusive end date.
    pub fn new(start: NaiveDate, end: NaiveDate) -> DateRange {
        DateRange { start, end }
    }
    /// Returns `true` if the *from* and *to* dates are equal.
    pub fn is_one_day(&self) -> bool {
        &self.start == &self.end
    }
    /// Returns `true` if the date range covers an entire year.
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
    /// Returns `true` if the date range covers multi entire years.
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
    /// Identifies if a date is within the date range.
    ///
    /// # Arguments
    ///
    /// * `date` is the date that will be checked.
    pub fn contains(&self, date: &NaiveDate) -> bool {
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
impl std::fmt::Display for DateRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt: &'static str = "%b-%d-%Y";
        if self.is_one_day() {
            write!(f, "{}", self.start.format(fmt))
        } else if self.is_one_year() {
            write!(f, "{:04}", self.start.year())
        } else if self.is_multi_year() {
            write!(f, "{:04} thru {:04}", self.start.year(), self.end.year())
        } else {
            write!(f, "{} thru {}", self.start.format(fmt), self.end.format(fmt))
        }
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
    fn date_range_is_within() {
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
        macro_rules! date_range {
            ($start:expr, $end:expr) => {
                DateRange::new(get_date($start.0, $start.1, $start.2), get_date($end.0, $end.1, $end.2))
            };
        }
        assert_eq!(date_range!((2020, 1, 1), (2020, 1, 1)).to_string(), "Jan-01-2020");
        assert_eq!(date_range!((2020, 1, 1), (2020, 12, 31)).to_string(), "2020");
        assert_eq!(date_range!((2020, 1, 1), (2021, 12, 31)).to_string(), "2020 thru 2021");
        assert_eq!(date_range!((2020, 1, 1), (2022, 1, 1)).to_string(), "Jan-01-2020 thru Jan-01-2022");
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
