//! This library defines and implements `PyO3` bindings that can be used by Python applications.
//!
//! There is currently no support for administration tasks such as initialization. There is support
//! to add locations, retrieve history, and query history. The library include Python type checking metadata.
//! Type checks are included for the weather data API and Python based entities.
//!
mod py_entities;
mod py_weather_data;

use pyo3::prelude::*;

/// Create errors returned from weather data as system errors.
macro_rules! system_err {
    ($error:expr) => {
        Err(pyo3::exceptions::PySystemError::new_err($error.to_string()))
    };
}
use system_err;

/// The `Python` weather data classes and functions.
///
#[pymodule]
fn py_weather_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<py_entities::PyWeatherConfig>()?;
    m.add_class::<py_entities::PyLocation>()?;
    m.add_class::<py_entities::PyHistory>()?;
    m.add_class::<py_entities::PyDailyHistories>()?;
    m.add_class::<py_entities::PyDateRange>()?;
    m.add_class::<py_entities::PyHistoryDates>()?;
    m.add_class::<py_entities::PyHistorySummary>()?;
    m.add_class::<py_entities::PyFilesysHistorySummary>()?;
    m.add_class::<py_entities::PyDatabaseHistorySummary>()?;
    m.add_class::<py_entities::PyLocationFilter>()?;
    m.add_class::<py_entities::PyLocationFilters>()?;
    m.add_class::<py_entities::PyHistoriesFuture>()?;
    m.add_function(wrap_pyfunction!(py_weather_data::create, m)?)?;
    m.add_class::<py_weather_data::PyWeatherData>()?;
    Ok(())
}
