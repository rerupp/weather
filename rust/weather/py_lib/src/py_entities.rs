//! The weather data to Python class mappings.

use super::*;
use chrono::prelude::{NaiveDate, NaiveDateTime};
use std::path::PathBuf;
use weather_lib::prelude::{
    CityFilter, DailyHistories, DateRange, HistoriesFuture, History, HistoryDates, HistorySummaries, Location,
    LocationFilter, State,
};

#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all, from_py_object)]
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
#[pyclass(get_all, set_all, from_py_object)]
pub struct PyLocation {
    /// The location country name.
    pub country_name: String,
    /// The location country code.
    pub country_code: String,
    /// The location region name.
    pub region_name: String,
    /// The location region code.
    pub region_code: String,
    /// The location city name.
    pub city_name: String,
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
macro_rules! map_location {
    ($location: ident) => {
        Self {
            country_name: $location.country_name,
            country_code: $location.country_code,
            region_name: $location.region_name,
            region_code: $location.region_code,
            city_name: $location.city_name,
            alias: $location.alias,
            longitude: $location.longitude,
            latitude: $location.latitude,
            tz: $location.tz,
        }
    };
}
impl From<Location> for PyLocation {
    fn from(location: Location) -> Self {
        map_location!(location)
    }
}
impl From<PyLocation> for Location {
    fn from(location: PyLocation) -> Self {
        map_location!(location)
    }
}
#[pymethods]
impl PyLocation {
    #[new]
    #[pyo3(signature = (city_name=None, country_name=None, country_code=None, region_name=None, region_code=None, alias=None, latitude=None, longitude=None, tz=None))]
    fn new(
        city_name: Option<String>,
        country_name: Option<String>,
        country_code: Option<String>,
        region_name: Option<String>,
        region_code: Option<String>,
        alias: Option<String>,
        latitude: Option<String>,
        longitude: Option<String>,
        tz: Option<String>,
    ) -> Self {
        Self {
            city_name: city_name.unwrap_or_default().trim().to_string(),
            country_name: country_name.unwrap_or_default().trim().to_string(),
            country_code: country_code.unwrap_or_default().trim().to_string(),
            region_name: region_name.unwrap_or_default().trim().to_string(),
            region_code: region_code.unwrap_or_default().trim().to_string(),
            alias: alias.unwrap_or_default().trim().to_string(),
            latitude: latitude.unwrap_or_default().trim().to_string(),
            longitude: longitude.unwrap_or_default().trim().to_string(),
            tz: tz.unwrap_or_default().trim().to_string(),
        }
    }
    fn __str__(&self) -> String {
        format!("{} ({})", self.city_name, self.region_code)
    }
    fn __copy__(&self) -> PyLocation {
        self.clone()
    }
}

/// The weather history data.
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all, from_py_object)]
// RustRover unfortunately sees PyHistory as a duplicate of History
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
macro_rules! map_history {
    ($history: ident) => {
        Self {
            alias: $history.alias,
            date: $history.date,
            temperature_high: $history.temperature_high,
            temperature_low: $history.temperature_low,
            temperature_mean: $history.temperature_mean,
            dew_point: $history.dew_point,
            humidity: $history.humidity,
            precipitation_chance: $history.precipitation_chance,
            precipitation_type: $history.precipitation_type,
            precipitation_amount: $history.precipitation_amount,
            wind_speed: $history.wind_speed,
            wind_gust: $history.wind_gust,
            wind_direction: $history.wind_direction,
            cloud_cover: $history.cloud_cover,
            pressure: $history.pressure,
            uv_index: $history.uv_index,
            sunrise: $history.sunrise,
            sunset: $history.sunset,
            moon_phase: $history.moon_phase,
            visibility: $history.visibility,
            description: $history.description,
        }
    };
}
impl From<History> for PyHistory {
    fn from(history: History) -> Self {
        map_history!(history)
    }
}
impl From<PyHistory> for History {
    fn from(location: PyHistory) -> Self {
        map_history!(location)
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
    fn wind_direction_str(&self) -> String {
        History::wind_direction_str(self.wind_direction).to_string()
    }
    fn uv_index_str(&self) -> String {
        History::uv_index_str(self.uv_index).to_string()
    }
    fn moon_phase_str(&self) -> String {
        History::moon_phase_str(self.moon_phase).to_string()
    }
}

/// A locations daily weather history.
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all, from_py_object)]
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

#[derive(Debug)]
#[pyclass]
pub struct PyHistoriesFuture {
    future: HistoriesFuture,
}
impl PyHistoriesFuture {
    pub fn new(future: HistoriesFuture) -> Self {
        Self { future }
    }
}
#[pymethods]
impl PyHistoriesFuture {
    pub fn is_finished(&self) -> bool {
        self.future.is_finished()
    }
    pub fn get(&self) -> PyResult<PyDailyHistories> {
        match self.future.get() {
            Err(error) => system_err!(error),
            Ok(maybe_histories) => match maybe_histories {
                None => system_err!("There were no histories returned."),
                Some(daily_histories) => Ok(PyDailyHistories::from(daily_histories)),
            },
        }
    }
}

#[derive(Clone, Debug)]
#[pyclass(from_py_object)]
pub struct PyDateRange {
    inner: DateRange,
}
impl From<DateRange> for PyDateRange {
    fn from(date_range: DateRange) -> Self {
        Self { inner: date_range }
    }
}
impl From<PyDateRange> for DateRange {
    fn from(date_range: PyDateRange) -> Self {
        Self { start: date_range.inner.start, end: date_range.inner.end }
    }
}
#[pymethods]
impl PyDateRange {
    #[new]
    fn new(start: NaiveDate, end: NaiveDate) -> PyResult<Self> {
        match start > end {
            true => Err(pyo3::exceptions::PyValueError::new_err("start date is after end date")),
            false => Ok(Self { inner: DateRange::new(start, end) }),
        }
    }
    #[getter]
    fn get_start(&self) -> NaiveDate {
        self.inner.start
    }
    #[setter]
    fn set_start(&mut self, start: NaiveDate) -> PyResult<()> {
        match start > self.inner.end {
            true => Err(pyo3::exceptions::PyValueError::new_err("start date is after end date")),
            false => {
                self.inner.start = start;
                Ok(())
            }
        }
        // self.inner.start = start;
        // Ok(())
    }
    #[getter]
    fn get_end(&self) -> NaiveDate {
        self.inner.end
    }
    #[setter]
    fn set_end(&mut self, end: NaiveDate) -> PyResult<()> {
        match end < self.inner.start {
            true => Err(pyo3::exceptions::PyValueError::new_err("end date is before start date")),
            false => {
                self.inner.end = end;
                Ok(())
            }
        }
    }
    fn __str__(&self) -> String {
        self.inner.to_string()
    }
    fn __copy__(&self) -> PyDateRange {
        PyDateRange { inner: self.inner.clone() }
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.inner.start == other.inner.start && self.inner.end == other.inner.end
    }
    fn annualized(&self) -> Vec<PyDateRange> {
        self.inner.annualized().into_iter().map(|daterange| daterange.into()).collect()
    }
    fn contains(&self, date: NaiveDate) -> bool {
        self.inner.contains(&date)
    }
    fn is_one_day(&self) -> bool {
        self.inner.is_one_day()
    }
    fn is_one_year(&self) -> bool {
        self.inner.is_one_year()
    }
    fn is_multi_year(&self) -> bool {
        self.inner.is_multi_year()
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

/// The data structure used to get locations.
///
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all, from_py_object)]
pub struct PyLocationFilter {
    /// A location can be searched for by its alias.
    pub alias: Option<String>,
    /// A location can be searched for by its name.
    pub city: Option<String>,
    /// A location can be searched by the region name or code.
    pub region: Option<String>,
    /// A location can be searched by the country name or code.
    pub country: Option<String>,
}
impl From<PyLocationFilter> for LocationFilter {
    fn from(filter: PyLocationFilter) -> Self {
        Self { alias: filter.alias, city: filter.city, region: filter.region, country: filter.country }
    }
}
#[pymethods]
impl PyLocationFilter {
    #[new]
    #[pyo3(signature = (alias=None, city=None, region=None, country=None))]
    pub fn new(alias: Option<String>, city: Option<String>, region: Option<String>, country: Option<String>) -> Self {
        Self { alias, city, region, country }
    }
    fn ___str__(&self) -> String {
        format!("{:?}", self)
    }
}

/// The collection of filters used to select locations.
///
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all, from_py_object)]
pub struct PyLocationFilters {
    pub filters: Vec<PyLocationFilter>,
}
impl From<PyLocationFilters> for Option<Vec<LocationFilter>> {
    fn from(filters: PyLocationFilters) -> Self {
        match filters.filters.len() {
            0 => None,
            _ => Some(filters.filters.into_iter().map(Into::into).collect()),
        }
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
#[pyclass(get_all, set_all, from_py_object)]
pub struct PyCityFilter {
    /// The optional city name.
    pub name: Option<String>,
    /// The optional region name.
    pub region: Option<String>,
    /// The optional country name.
    pub country: Option<String>,
    /// Limits the number of matches that will be returned.
    pub limit: usize,
}
impl From<PyCityFilter> for CityFilter {
    fn from(py_filter: PyCityFilter) -> Self {
        Self { name: py_filter.name, region: py_filter.region, country: py_filter.country, limit: py_filter.limit }
    }
}
#[pymethods]
impl PyCityFilter {
    #[new]
    #[pyo3(signature = (name=None, region=None, country=None, limit=25))]
    fn new(name: Option<String>, region: Option<String>, country: Option<String>, limit: Option<usize>) -> Self {
        Self { name, region, country, limit: limit.unwrap_or(25) }
    }
    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

/// The US City state names.
#[derive(Clone, Debug, Default)]
#[pyclass(get_all, set_all, from_py_object)]
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
