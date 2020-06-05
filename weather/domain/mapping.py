from datetime import datetime, date
from decimal import Decimal
from enum import Enum
from typing import Dict, Callable, TypeVar, Generic, Sequence, Any


class HourlyWeatherContent(Enum):
    TIME = "time"
    TEMPERATURE = "temperature"
    APPARENT_TEMPERATURE = "apparentTemperature"
    WIND_SPEED = "windSpeed"
    WIND_GUST = "windGust"
    WIND_BEARING = "windBearing"
    CLOUD_COVER = "cloudCover"


class DailyWeatherContent(Enum):
    TIME = "time"
    TEMPERATURE_HIGH = "temperatureHigh"
    TEMPERATURE_HIGH_TIME = "temperatureHighTime"
    TEMPERATURE_LOW = "temperatureLow"
    TEMPERATURE_LOW_TIME = "temperatureLowTime"
    TEMPERATURE_MAX = "temperatureMax"
    TEMPERATURE_MAX_TIME = "temperatureMaxTime"
    TEMPERATURE_MIN = "temperatureMin"
    TEMPERATURE_MIN_TIME = "temperatureMinTime"
    WIND_SPEED = "windSpeed"
    WIND_GUST = "windGust"
    WIND_GUST_TIME = "windGustTime"
    WIND_BEARING = "windBearing"
    CLOUD_COVER = "cloudCover"


class DataConverter:
    """Collect all the standard data converters into one namespace"""

    @staticmethod
    def to_str(data: any) -> str:
        if data:
            return str(data)

    @staticmethod
    def to_date(data: any, tz: datetime.tzinfo, fmt: str = None) -> str:
        return DataConverter.convert_timestamp(data, tz, fmt if fmt else "%Y-%m-%d")

    @staticmethod
    def to_datetime(data: any, tz: datetime.tzinfo, fmt: str = None) -> str:
        return DataConverter.convert_timestamp(data, tz, fmt if fmt else "%Y-%m-%d %H:%M:%S")

    @staticmethod
    def to_time(data: any, tz: datetime.tzinfo, fmt: str = None) -> str:
        return DataConverter.convert_timestamp(data, tz, fmt if fmt else "%H:%M:%S")

    @staticmethod
    def convert_timestamp(data: any, tz: datetime.tzinfo, fmt: str) -> str:
        if data:
            return DataConverter.to_binary_datetime(data, tz).strftime(fmt)

    @staticmethod
    def to_binary_date(data: any, tz: datetime.tzinfo) -> date:
        if data:
            return DataConverter.to_binary_datetime(data, tz).date()

    @staticmethod
    def to_binary_datetime(data: any, tz: datetime.tzinfo) -> datetime:
        if data:
            ts = data if isinstance(data, int) else int(data)
            return datetime.fromtimestamp(ts, tz=tz)

    _compass_bearing = ["N", "NNE", "NE", "ENE",
                        "S", "SSW", "SW", "WSW",
                        "E", "ESE", "SE", "SSE",
                        "W", "WNW", "NW", "NNW"]

    @staticmethod
    def wind_bearing(data: any) -> str:
        if data:
            direction_index = int((data / 22.5) + .5)
            return DataConverter._compass_bearing[direction_index % 16]

    @staticmethod
    def to_fahrenheit(data: any) -> str:
        if data:
            return "{:.1f}".format(Decimal(data))

    @staticmethod
    def to_float(data: Any) -> float:
        if data:
            return float(data)


ConvertType = TypeVar("ConvertType", str, Enum)


class GenericDataConverter(Generic[ConvertType]):

    def __init__(self, translators: Dict[ConvertType, Callable[[Any], Any]]):
        self._translators = translators

    def convert_contents(self,
                         data_dict: Dict[str, Any],
                         content_selection: Sequence[ConvertType]) -> Dict[ConvertType, Any]:
        translators = self._translators
        results: Dict[ConvertType, Any] = {}
        for field in content_selection:
            data = data_dict.get(field.value if isinstance(field, Enum) else field)
            if data:
                convert_data = translators.get(field)
                if not convert_data:
                    raise ValueError("Converts action for '{}' not found...".format(field))
                data = convert_data(data)
            # always add the field regardless if there's data
            results[field] = data
        return results
