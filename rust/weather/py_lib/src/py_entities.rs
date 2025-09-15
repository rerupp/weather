//! The weather data to Python class mappings.

use super::*;
use chrono::prelude::{NaiveDate, NaiveDateTime};
use std::path::PathBuf;
use weather_lib::prelude::{
    CityFilter, DailyHistories, DateRange, History, HistoryDates, HistorySummaries, Location, LocationFilter,
    LocationFilters, State
};

#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all)]
pub struct PyWeatherConfig {
    pub config_file: Option<PathBuf>,
    pub dirname: Option<PathBuf>,
    pub logfile: Option<PathBuf>,
    pub log_append: bool,
    pub log_level: usize,
    pub fs_only: bool,
}
#[pymethods]
impl PyWeatherConfig {
    #[new]
    #[pyo3(signature = (config_file=None, dirname=None, logfile=None, log_append=false, log_level=0, fs_only=false))]
    fn new(
        config_file: Option<PathBuf>,
        dirname: Option<PathBuf>,
        logfile: Option<PathBuf>,
        log_append: bool,
        log_level: usize,
        fs_only: bool,
    ) -> Self {
        Self { config_file, dirname, logfile, log_append, log_level, fs_only }
    }
}

/// The `Python` data that comprises a location.
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all)]
pub struct PyLocation {
    /// The location city name.
    pub city: String,
    /// The locations full state name.
    pub state: String,
    /// The locations two-letter abbreviation.
    pub state_id: String,
    /// The name of a location.
    pub name: String,
    /// A unique nickname of a location.
    pub alias: String,
    /// The location longitude.
    pub longitude: String,
    /// The location latitude.
    pub latitude: String,
    /// the location timezone.
    pub tz: String,
}
impl From<&Location> for PyLocation {
    fn from(location: &Location) -> Self {
        location.clone().into()
    }
}
impl From<Location> for PyLocation {
    fn from(location: Location) -> Self {
        Self {
            city: location.city,
            state: location.state,
            state_id: location.state_id,
            name: location.name,
            alias: location.alias,
            longitude: location.longitude,
            latitude: location.latitude,
            tz: location.tz,
        }
    }
}
impl From<PyLocation> for Location {
    fn from(location: PyLocation) -> Self {
        Self {
            city: location.city,
            state: location.state,
            state_id: location.state_id,
            name: location.name,
            alias: location.alias,
            longitude: location.longitude,
            latitude: location.latitude,
            tz: location.tz,
        }
    }
}
#[pymethods]
impl PyLocation {
    #[new]
    #[pyo3(signature = (city=None, state=None, state_id=None, alias=None, latitude=None, longitude=None, tz=None))]
    fn new(
        city: Option<String>,
        state: Option<String>,
        state_id: Option<String>,
        alias: Option<String>,
        latitude: Option<String>,
        longitude: Option<String>,
        tz: Option<String>,
    ) -> Self {
        let city = city.unwrap_or(Default::default()).trim().to_string();
        let state_id = state_id.unwrap_or(Default::default()).trim().to_string();
        let name = if city.is_empty() && state_id.is_empty() {
            Default::default()
        } else {
            format!("{city}, {state_id}")
        };
        Self {
            city,
            state: state.unwrap_or(Default::default()).trim().to_string(),
            state_id,
            name,
            alias: alias.unwrap_or(Default::default()),
            latitude: latitude.unwrap_or(Default::default()).trim().to_string(),
            longitude: longitude.unwrap_or(Default::default()).trim().to_string(),
            tz: tz.unwrap_or(Default::default()).trim().to_string(),
        }
    }
    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
    fn __copy__(&self) -> PyLocation {
        PyLocation::new(
            Some(self.city.clone()),
            Some(self.state.clone()),
            Some(self.state_id.clone()),
            Some(self.alias.clone()),
            Some(self.latitude.clone()),
            Some(self.longitude.clone()),
            Some(self.tz.clone()),
        )
    }
}

/// The weather history data.
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all)]
pub struct PyHistory {
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
impl From<History> for PyHistory {
    fn from(history: History) -> Self {
        Self {
            alias: history.alias,
            date: history.date,
            temperature_high: history.temperature_high,
            temperature_low: history.temperature_low,
            temperature_mean: history.temperature_mean,
            dew_point: history.dew_point,
            humidity: history.humidity,
            precipitation_chance: history.precipitation_chance,
            precipitation_type: history.precipitation_type,
            precipitation_amount: history.precipitation_amount,
            wind_speed: history.wind_speed,
            wind_gust: history.wind_gust,
            wind_direction: history.wind_direction,
            cloud_cover: history.cloud_cover,
            pressure: history.pressure,
            uv_index: history.uv_index,
            sunrise: history.sunrise,
            sunset: history.sunset,
            moon_phase: history.moon_phase,
            visibility: history.visibility,
            description: history.description,
        }
    }
}
impl From<PyHistory> for History {
    fn from(location: PyHistory) -> Self {
        Self {
            alias: location.alias,
            date: location.date,
            temperature_high: location.temperature_high,
            temperature_low: location.temperature_low,
            temperature_mean: location.temperature_mean,
            dew_point: location.dew_point,
            humidity: location.humidity,
            precipitation_chance: location.precipitation_chance,
            precipitation_type: location.precipitation_type,
            precipitation_amount: location.precipitation_amount,
            wind_speed: location.wind_speed,
            wind_gust: location.wind_gust,
            wind_direction: location.wind_direction,
            cloud_cover: location.cloud_cover,
            pressure: location.pressure,
            uv_index: location.uv_index,
            sunrise: location.sunrise,
            sunset: location.sunset,
            moon_phase: location.moon_phase,
            visibility: location.visibility,
            description: location.description,
        }
    }
}
#[pymethods]
impl PyHistory {
    #[new]
    fn new() -> Self {
        Default::default()
    }
    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

/// A locations daily weather history.
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all)]
pub struct PyDailyHistories {
    /// The location metadata.
    pub location: PyLocation,
    /// The daily histories for a location.
    pub histories: Vec<PyHistory>,
}
impl From<DailyHistories> for PyDailyHistories {
    fn from(daily_histories: DailyHistories) -> Self {
        Self {
            location: daily_histories.location.into(),
            histories: daily_histories.histories.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<PyDailyHistories> for DailyHistories {
    fn from(daily_histories: PyDailyHistories) -> Self {
        Self {
            location: daily_histories.location.into(),
            histories: daily_histories.histories.into_iter().map(Into::into).collect(),
        }
    }
}
#[pymethods]
impl PyDailyHistories {
    #[new]
    fn __new__() -> Self {
        Default::default()
    }
    fn __str__(&self) -> String {
        let mut str = vec![];
        str.push("DailyHistories {".to_string());
        str.push(format!("  location: {:?}", self.location));
        str.push("  histories: [".to_string());
        self.histories.iter().for_each(|history| str.push(format!("  {:?}", history)));
        str.push("  ]".to_string());
        str.push("}".to_string());
        str.join("\n")
    }
}

/// The container for a range of dates.
#[derive(Clone, Debug)]
#[pyclass(get_all, set_all)]
pub struct PyDateRange {
    /// The starting date of the range.
    pub start: NaiveDate,
    /// The inclusive end date of the range.
    pub end: NaiveDate,
}
impl From<DateRange> for PyDateRange {
    fn from(date_range: DateRange) -> Self {
        Self { start: date_range.start, end: date_range.end }
    }
}
impl From<PyDateRange> for DateRange {
    fn from(date_range: PyDateRange) -> Self {
        Self { start: date_range.start, end: date_range.end }
    }
}
#[pymethods]
impl PyDateRange {
    #[new]
    fn new(start: NaiveDate, end: NaiveDate) -> PyResult<Self> {
        match start > end {
            true => Err(pyo3::exceptions::PyValueError::new_err("from date greater than to date")),
            false => Ok(Self { start, end }),
        }
    }
    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
    fn __copy__(&self) -> PyDateRange {
        PyDateRange::new(self.start, self.end).unwrap()
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
    fn contains(&self, date: NaiveDate) -> PyResult<bool> {
        Ok(self.start <= date && date <= self.end)
    }
}

#[derive(Debug, Default)]
#[pyclass(get_all)]
pub struct PyHistoryDates {
    /// The location metadata.
    pub location: PyLocation,
    /// The history dates metadata.
    pub history_dates: Vec<PyDateRange>,
}
impl From<HistoryDates> for PyHistoryDates {
    fn from(history_dates: HistoryDates) -> Self {
        Self {
            location: history_dates.location.clone().into(),
            history_dates: history_dates.history_dates.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Default)]
#[pyclass(get_all)]
/// A locations history summary.
pub struct PyHistorySummaries {
    location: PyLocation,
    /// The number of weather data histories available.
    count: usize,
    /// The overall size of weather data in bytes (may or may not be available).
    overall_size: Option<usize>,
    /// The size in bytes of weather data.
    raw_size: Option<usize>,
    /// The size in bytes of weather data in the backing store.
    store_size: Option<usize>,
}
impl From<HistorySummaries> for PyHistorySummaries {
    fn from(history_summaries: HistorySummaries) -> Self {
        Self {
            location: history_summaries.location.into(),
            count: history_summaries.count,
            overall_size: history_summaries.overall_size,
            raw_size: history_summaries.raw_size,
            store_size: history_summaries.store_size,
        }
    }
}
#[pymethods]
impl PyHistorySummaries {
    #[new]
    fn new() -> Self {
        Default::default()
    }
    fn ___str__(&self) -> String {
        format!("{:?}", self)
    }
}

/// The data structure used to get locations..
///
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all)]
pub struct PyLocationFilter {
    /// A location can be searched by the city name.
    pub city: Option<String>,

    /// A location can be searched by the state name (full or two-letter form).
    pub state: Option<String>,

    /// A location can be searched for by its name or alias.
    pub name: Option<String>,
}
impl From<PyLocationFilter> for LocationFilter {
    fn from(py_filter: PyLocationFilter) -> Self {
        LocationFilter { city: py_filter.city, state: py_filter.state, name: py_filter.name }
    }
}
#[pymethods]
impl PyLocationFilter {
    #[new]
    #[pyo3(signature = (city=None, state=None, name=None))]
    pub fn new(city: Option<String>, state: Option<String>, name: Option<String>) -> Self {
        Self { city, state, name }
    }
    fn ___str__(&self) -> String {
        format!("{:?}", self)
    }
}

/// The collection of filters used to select locations.
///
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all)]
pub struct PyLocationFilters {
    pub filters: Vec<PyLocationFilter>,
}
impl From<PyLocationFilters> for LocationFilters {
    fn from(py_filters: PyLocationFilters) -> Self {
        let filters = py_filters.filters.into_iter().map(Into::into).collect();
        LocationFilters::new(filters)
    }
}
#[pymethods]
impl PyLocationFilters {
    #[new]
    #[pyo3(signature = (filters=vec![]))]
    pub fn new(filters: Vec<PyLocationFilter>) -> Self {
        Self { filters }
    }
    fn ___str__(&self) -> String {
        format!("{:?}", self)
    }
}

/// The filter used to select US cities.
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all)]
pub struct PyCityFilter {
    /// The optional city name.
    pub name: Option<String>,

    /// The optional state name.
    pub state: Option<String>,

    /// The optional zip code.
    pub zip_code: Option<String>,

    /// Limits the number of matches that will be returned.
    pub limit: usize,
}
impl From<PyCityFilter> for CityFilter {
    fn from(py_filter: PyCityFilter) -> Self {
        Self { name: py_filter.name, state: py_filter.state, zip_code: py_filter.zip_code, limit: py_filter.limit }
    }
}
#[pymethods]
impl PyCityFilter {
    #[new]
    #[pyo3(signature = (name=None, state=None, zip_code=None, limit=25))]
    fn new(name: Option<String>, state: Option<String>, zip_code: Option<String>, limit: Option<usize>) -> Self {
        Self { name, state, zip_code, limit: limit.unwrap_or(25) }
    }
    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

/// The US City state names.
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all)]
pub struct PyState {
    /// The states full name.
    pub name: String,

    /// The two letter state abbreviation.
    pub state_id: String,
}
impl From<State> for PyState {
    fn from(state: State) -> Self {
        PyState { name: state.name, state_id: state.state_id }
    }
}
