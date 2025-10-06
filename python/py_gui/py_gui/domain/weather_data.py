from py_weather_lib import PyWeatherData
class WeatherData:
    """
    Plumbing signatures through pyo3 is a PITA right now due to it requiring .pyi files.
    Since I'm the sole consumer of the Rust bindings this is easier to maintain than
    interface files.
    """

    def __init__(self, backend: PyWeatherData = None):
        self._backend = backend

    @property
    def backend(self) -> PyWeatherData:
        return self._backend
