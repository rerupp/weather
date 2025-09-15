# from typing import List
#
# import py_weather_lib as wd
# from py_weather_lib import (CityFilter, DailyHistories, DateRange, HistoryClient, HistorySummaries, Location,
#                             LocationFilter, LocationFilters, LocationHistoryDates)
#
# __all__ = ['WeatherData', 'DailyHistories', 'DateRange', 'HistoryClient', 'HistorySummaries', 'Location',
#            'LocationCriteria', 'LocationFilter', 'LocationFilters', 'LocationHistoryDates']
#
#
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
#
#     def add_histories(self, daily_histories: DailyHistories) -> int:
#         return self._rust_bindings.add_histories(daily_histories)
#
#     def get_history_client(self) -> HistoryClient:
#         return self._rust_bindings.get_history_client()
#
#     def get_daily_history(self, filter: LocationFilter, history_range: DateRange) -> DailyHistories:
#         return self._rust_bindings.get_daily_history(filter, history_range)
#
#     def get_history_dates(self, filters=LocationFilters()) -> List[LocationHistoryDates]:
#         return self._rust_bindings.get_history_dates(filters)
#
#     def get_history_summary(self, filters=LocationFilters()) -> List[HistorySummaries]:
#         return self._rust_bindings.get_history_summary(filters)
#
#     def get_locations(self, filters=LocationFilters()) -> List[Location]:
#         return self._rust_bindings.get_locations(filters)
#
#     def search_locations(self, filter: CityFilter) -> List[Location]:
#         return self._rust_bindings.search_locations(filter)
#
#     def add_location(self, location: Location):
#         self._rust_bindings.add_location(location)
