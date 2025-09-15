# from .weather_data import (CityFilter, DailyHistories, DateRange, HistoryClient, HistorySummaries, Location,
#                            LocationFilter, LocationFilters, LocationHistoryDates, WeatherData)

from .weather_data import WeatherData

class WeatherConfigException(Exception):
    def __init__(self, message: str):
        self.add_note(message)

    def reason(self):
        '\n'.join(self.__notes__)


__all__ = ['WeatherData', 'WeatherConfigException']
