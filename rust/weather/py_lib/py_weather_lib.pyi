# Unfortunately the typing information is needed to make sure PyCharm can grok
# PyO3 bindings. It doesn't have functionality that lets it open and view the
# native library to understand function signatures. My guess is Jet Brains take
# on this is typing files are the recommended way to do this for Python. This
# could turn into  maintenance nightmare however it is nice having the Python
# code understand the interface.
from datetime import date, datetime
from typing import List


def create(config: PyWeatherConfig) -> PyWeatherData: ...


class PyWeatherData:
    def add_histories(self, daily_histories: PyDailyHistories) -> int: ...

    def get_history_client(self) -> PyHistoryClient: ...

    def get_daily_history(self, filter: PyLocationFilter, date_range: PyDateRange) -> PyDailyHistories: ...

    def get_history_dates(self, filters: PyLocationFilters) -> List[PyHistoryDates]: ...

    def get_history_summary(self, filters: PyLocationFilters) -> List[PyHistorySummaries]: ...

    def get_locations(self, filters: PyLocationFilters) -> List[PyLocation]: ...

    def add_location(self, location: PyLocation) -> None: ...

    def search_locations(self, filter: PyCityFilter) -> List[PyLocation]: ...

    def get_states(self) -> List[PyState]: ...


class PyWeatherConfig:
    @property
    def config_file(self) -> str | None: ...

    @config_file.setter
    def config_file(self, str) -> None: ...

    @property
    def dirname(self) -> str | None: ...

    @dirname.setter
    def dirname(self, str) -> None: ...

    @property
    def logfile(self) -> str | None: ...

    @logfile.setter
    def logfile(self, str) -> None: ...

    @property
    def log_append(self) -> bool: ...

    @log_append.setter
    def logfile(self, bool) -> None: ...

    @property
    def log_level(self) -> int: ...

    @log_level.setter
    def log_level(self, int) -> None: ...

    @property
    def fs_only(self) -> bool: ...

    @fs_only.setter
    def fs_only(self, bool) -> None: ...

    def __new__(
            cls,
            config_file: str | None = None,
            dirname: str | None = None,
            logfile: str | None = None,
            log_append=False,
            log_level=0,
            fs_only=False,
    ) -> PyWeatherConfig: ...


class PyHistoryClient:
    def execute(self, location: PyLocation, date_range: PyDateRange) -> None: ...

    def poll(self) -> bool: ...

    def get(self) -> PyDailyHistories: ...


class PyLocation:
    @property
    def city(self) -> str | None: ...

    @city.setter
    def city(self, str) -> None: ...

    @property
    def state(self) -> str | None: ...

    @state.setter
    def state(self, str) -> None: ...

    @property
    def state_id(self) -> str | None: ...

    @state_id.setter
    def state_id(self, str) -> None: ...

    @property
    def name(self) -> str | None: ...

    @property
    def alias(self) -> str | None: ...

    @alias.setter
    def alias(self, str) -> None: ...

    @property
    def longitude(self) -> str | None: ...

    @longitude.setter
    def longitude(self, str) -> None: ...

    @property
    def latitude(self) -> str | None: ...

    @latitude.setter
    def latitude(self, str) -> None: ...

    @property
    def tz(self) -> str | None: ...

    @tz.setter
    def tz(self, str) -> None: ...

    def __new__(
            cls,
            city: str | None = None,
            state: str | None = None,
            state_id: str | None = None,
            alias: str | None = None,
            latitude: str | None = None,
            longitude: str | None = None,
            tz: str | None = None,
    ) -> PyLocation: ...


class PyHistory:
    @property
    def alias(self) -> str: ...

    @alias.setter
    def alias(self, str) -> None: ...

    @property
    def date(self) -> date: ...

    @date.setter
    def alias(self, date) -> None: ...

    @property
    def temperature_high(self) -> float | None: ...

    @temperature_high.setter
    def temperature_high(self, high: float | None) -> None: ...

    @property
    def temperature_low(self) -> float | None: ...

    @temperature_low.setter
    def temperature_low(self, low: float | None) -> None: ...

    @property
    def temperature_mean(self) -> float | None: ...

    @temperature_mean.setter
    def temperature_mean(self, mean: float | None) -> None: ...

    @property
    def dew_point(self) -> float | None: ...

    @dew_point.setter
    def dew_point(self, dew_point: float | None) -> None: ...

    @property
    def humidity(self) -> float | None: ...

    @humidity.setter
    def humidity(self, humidity: float | None) -> None: ...

    @property
    def precipitation_chance(self) -> float | None: ...

    @precipitation_chance.setter
    def precipitation_chance(self, precip_chance: float | None) -> None: ...

    @property
    def precipitation_type(self) -> str | None: ...

    @precipitation_type.setter
    def precipitation_type(self, precip_type: str | None) -> None: ...

    @property
    def precipitation_amount(self) -> float | None: ...

    @precipitation_amount.setter
    def precipitation_amount(self, precip_amount: float | None) -> None: ...

    @property
    def wind_speed(self) -> float | None: ...

    @wind_speed.setter
    def wind_speed(self, speed: float | None) -> None: ...

    @property
    def wind_gust(self) -> float | None: ...

    @wind_gust.setter
    def wind_gust(self, gust: float | None) -> None: ...

    @property
    def wind_direction(self) -> float | None: ...

    @wind_direction.setter
    def wind_direction(self, direction: float | None) -> None: ...

    @property
    def cloud_cover(self) -> float | None: ...

    @cloud_cover.setter
    def cloud_cover(self, cover: float | None) -> None: ...

    @property
    def pressure(self) -> float | None: ...

    @pressure.setter
    def pressure(self, pressure: float | None) -> None: ...

    @property
    def uv_index(self) -> float | None: ...

    @uv_index.setter
    def uv_index(self, index: float | None) -> None: ...

    @property
    def sunrise(self) -> datetime | None: ...

    @sunrise.setter
    def sunrise(self, when: datetime | None) -> None: ...

    @property
    def sunset(self) -> datetime | None: ...

    @sunset.setter
    def sunset(self, when: datetime | None) -> None: ...

    @property
    def moon_phase(self) -> float | None: ...

    @moon_phase.setter
    def moon_phase(self, phase: float | None) -> None: ...

    @property
    def visibility(self) -> float | None: ...

    @visibility.setter
    def visibility(self, phase: float | None) -> None: ...

    @property
    def description(self) -> str | None: ...

    @description.setter
    def description(self, descr: str | None) -> None: ...

    def __new__(cls) -> PyHistory: ...


class PyDailyHistories:
    @property
    def location(self) -> PyLocation: ...

    @location.setter
    def location(self, location: PyLocation) -> None: ...

    @property
    def histories(self) -> List[PyHistory]: ...

    @histories.setter
    def histories(self, histories: List[PyHistory]) -> None: ...

    def __new__(cls) -> PyDailyHistories: ...


class PyDateRange:
    @property
    def start(self) -> date: ...

    @start.setter
    def start(self, start: date) -> None: ...

    @property
    def end(self) -> date: ...

    @end.setter
    def end(self, end: date) -> None: ...

    def __new__(cls, start: date, end: date) -> PyDateRange: ...

    def contains(self, date: date) -> bool: ...


class PyHistoryDates:
    @property
    def location(self) -> PyLocation: ...

    @property
    def history_dates(self) -> List[PyDateRange]: ...


class PyHistorySummaries:
    @property
    def location(self) -> PyLocation: ...

    @property
    def count(self) -> int | None: ...

    @property
    def overall_size(self) -> int | None: ...

    @property
    def raw_size(self) -> int | None: ...

    @property
    def store_size(self) -> int | None: ...

    def __new__(cls) -> PyHistorySummaries: ...


class PyLocationFilter:
    @property
    def city(self) -> str | None: ...

    @city.setter
    def city(self, str) -> None: ...

    @property
    def state(self) -> str | None: ...

    @state.setter
    def state(self, str) -> None: ...

    @property
    def name(self) -> str | None: ...

    @name.setter
    def name(self, str) -> None: ...

    def __new__(
            cls,
            city: str | None = None,
            state: str | None = None,
            name: str | None = None,
    ) -> PyLocationFilter: ...


class PyLocationFilters:
    @property
    def filters(self) -> List[PyLocationFilter]: ...

    @filters.setter
    def filters(self, filters: List[PyLocationFilter]) -> None: ...

    def __new__(
            cls,
            filters=List[PyLocationFilter],
    ) -> PyLocationFilters: ...


class PyCityFilter:
    @property
    def name(self) -> str | None: ...

    @name.setter
    def name(self, str) -> None: ...

    @property
    def state(self) -> str | None: ...

    @state.setter
    def state(self, str) -> None: ...

    @property
    def zip_code(self) -> str | None: ...

    @zip_code.setter
    def zip_code(self, str) -> None: ...

    @property
    def limit(self) -> int: ...

    @limit.setter
    def limit(self, int) -> None: ...

    def __new__(
            cls,
            name: str | None = None,
            state: str | None = None,
            zip_code: str | None = None,
            limit: int | None = None,
    ) -> PyCityFilter: ...


class PyState:
    @property
    def name(self) -> str | None: ...

    @name.setter
    def name(self, str) -> None: ...

    @property
    def state_id(self) -> str | None: ...

    @state_id.setter
    def state_id(self, str) -> None: ...
