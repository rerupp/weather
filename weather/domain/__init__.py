from logging import getLogger
from .mapping import DailyWeatherContent, HourlyWeatherContent, DataConverter, GenericDataConverter
from .objects import CityDB, CsvDictWriter, DateRange, DataPath, FullHistory, Location, DataPath, DictionaryWriter
from .models import (
    DataSourceReader, DataSourceWriter, WeatherHistory, HistoryProperties, WeatherData, WeatherHistoryProperties
)


_root_logger = getLogger(__name__)


def set_logging_level(level: int):
    _root_logger.setLevel(level)
