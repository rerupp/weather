from datetime import datetime, date
from enum import Enum
from typing import Dict, Callable, TypeVar, Generic, Sequence, Any, List


class HourlyWeatherContent(Enum):
    TIME = "time"
    TEMPERATURE = "temperature"
    APPARENT_TEMPERATURE = "apparentTemperature"
    WIND_SPEED = "windSpeed"
    WIND_GUST = "windGust"
    WIND_BEARING = "windBearing"
    CLOUD_COVER = "cloudCover"
    UV_INDEX = "uvIndex"
    SUMMARY = "summary"
    HUMIDITY = "humidity"
    DEW_POINT = "dewPoint"


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
    UV_INDEX = "uvIndex"
    UV_INDEX_TIME = "uvIndexTime"
    SUMMARY = "summary"
    HUMIDITY = "humidity"
    DEW_POINT = "dewPoint"
    SUNRISE_TIME = "sunriseTime"
    SUNSET_TIME = "sunsetTime"
    MOON_PHASE = "moonPhase"


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
        if data is not None:
            return DataConverter.to_binary_datetime(data, tz).strftime(fmt)

    @staticmethod
    def to_binary_date(data: any, tz: datetime.tzinfo) -> date:
        if data is not None:
            return DataConverter.to_binary_datetime(data, tz).date()

    @staticmethod
    def to_binary_datetime(data: any, tz: datetime.tzinfo) -> datetime:
        if data is not None:
            ts = data if isinstance(data, int) else int(data)
            return datetime.fromtimestamp(ts, tz=tz)

    _compass_bearing = ["N", "NNE", "NE", "ENE",
                        "S", "SSW", "SW", "WSW",
                        "E", "ESE", "SE", "SSE",
                        "W", "WNW", "NW", "NNW"]

    @staticmethod
    def wind_bearing(data: any) -> str:
        if data is not None:
            data = DataConverter.to_binary_float(data)
            direction_index = int((data / 22.5) + .5)
            return DataConverter._compass_bearing[direction_index % 16]

    @staticmethod
    def to_fahrenheit(data: any) -> str:
        return DataConverter.to_float(data, precision=1)

    @staticmethod
    def to_binary_float(data: Any) -> float:
        if data is not None:
            return data if isinstance(data, float) else float(data)

    @staticmethod
    def to_float(data: Any, precision: int = 2) -> str:
        if data is not None:
            data = DataConverter.to_binary_float(data)
            return "{:.{precision}f}".format(data, precision=precision)


ConvertType = TypeVar("ConvertType", str, Enum)


class GenericDataConverter(Generic[ConvertType]):

    def __init__(self, translators: Dict[ConvertType, Callable[[Any], Any]]):
        self._translators: Dict[str, Callable[[Any], Any]] = {}
        self._keys: List[ConvertType] = []
        for key, translator in translators.items():
            self._keys.append(key)
            self._translators[self.to_field_name(key)] = translator

    @staticmethod
    def to_field_name(key: ConvertType) -> str:
        return key.value if isinstance(key, Enum) else key

    def keys(self) -> List[ConvertType]:
        return self._keys.copy()

    def field_names(self) -> List[str]:
        return [self.to_field_name(key) for key in self._keys]

    def convert_contents(self,
                         data_dict: Dict[str, Any],
                         content_selection: Sequence[ConvertType]) -> Dict[ConvertType, Any]:
        translators = self._translators
        results: Dict[ConvertType, Any] = {}
        for field in content_selection:
            field_name = self.to_field_name(field)
            data = data_dict.get(field_name)
            if data is not None:
                convert_data = translators.get(field_name)
                if not convert_data:
                    raise ValueError("Convert action for '{}' not found...".format(field))
                data = convert_data(data)
            # always add the field regardless if there's data
            results[field] = data
        return results
