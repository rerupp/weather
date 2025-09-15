mod py_entities;
mod py_weather_data;
mod py_history_client;

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
    m.add_class::<py_entities::PyHistorySummaries>()?;
    m.add_class::<py_entities::PyLocationFilter>()?;
    m.add_class::<py_entities::PyLocationFilters>()?;
    m.add_class::<py_entities::PyCityFilter>()?;
    m.add_function(wrap_pyfunction!(py_weather_data::create, m)?)?;
    m.add_class::<py_weather_data::PyWeatherData>()?;
    m.add_class::<py_history_client::PyHistoryClient>()?;
    Ok(())
}
