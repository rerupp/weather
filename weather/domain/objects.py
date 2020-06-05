import logging
import sys
from calendar import monthrange
from contextlib import contextmanager
from csv import DictReader, DictWriter
from datetime import date, datetime, timedelta, MINYEAR
from enum import Enum
from gzip import GzipFile
from importlib.resources import open_binary as open_binary_package
from io import TextIOWrapper
from pathlib import Path
from re import compile as re_compile, IGNORECASE
from typing import Callable, Generator, IO, List, NamedTuple, Union
from urllib.parse import urlencode

import requests
from requests.compat import urljoin

from weather.configuration import get_setting

# bring the data package into scope
from weather.domain import data

DataPath = Union[str, Path]
DictionaryWriter = Callable[[dict], None]


class CsvDictWriter:

    def __init__(self, fields: List[str]):
        if not fields or len(fields) == 0:
            raise ValueError("Dictionary fields are required...")
        self._fields = fields.copy()

    @property
    def fields(self):
        return self._fields.copy()

    @contextmanager
    def file_writer(self, data_path: DataPath) -> DictionaryWriter:
        data_path = Path(data_path) if isinstance(data_path, str) else data_path
        if not data_path.exists():
            mode = "w"
        elif not data_path.is_file():
            raise ValueError("CSV filename exists and is not writable...")
        else:
            mode = "a"
        with data_path.open(mode) as fp:
            dict_writer = self._get_dict_writer(fp, mode == "w")
            yield lambda d: dict_writer.writerow(d)

    @contextmanager
    def stdout(self) -> DictionaryWriter:
        dict_writer = self._get_dict_writer(sys.stdout, True)
        yield lambda content: dict_writer.writerow(content)

    def _get_dict_writer(self, fp: IO, include_headers: bool = False) -> DictWriter:
        dict_writer = DictWriter(fp, fieldnames=self._fields, extrasaction='ignore')
        if include_headers:
            dict_writer.writeheader()
        return dict_writer


class DateRange(NamedTuple('_DateRange', low=date, high=date)):

    def __new__(cls, low: date, high: date = None):
        if not low:
            error = "{}: a low date is required.".format(cls.__name__)
            raise ValueError(error)
        if not high:
            high = low
        elif high < low:
            error = "{}: high date ({}) cannot be less than low date ({}).".format(cls.__name__, high, low)
            raise ValueError(error)
        # It looks like there is an open issue with PyCharm, PY-39755, that falsely reports
        # "unexpected arguments" when calling the super class.
        # noinspection PyArgumentList
        return super().__new__(cls, low, high)

    def __str__(self):
        return "{}(low={},high={})".format(self.__class__.__name__, self.low, self.high)

    def __eq__(self, other) -> bool:
        if isinstance(other, DateRange):
            return self.low == other.low and self.high == other.high
        raise NotImplemented

    def __contains__(self, other) -> bool:
        if isinstance(other, DateRange):
            return self.low <= other.low and self.high >= other.high

    def total_days(self) -> int:
        return (self.high - self.low).days

    def get_dates(self) -> Generator[date, None, None]:
        if self.low == self.high:
            yield self.low
        else:
            one_day = timedelta(days=1)
            ts = self.low
            while ts <= self.high:
                yield ts
                ts += one_day

    def spans_years(self) -> bool:
        return self.low.year < self.high.year

    def as_neutral_date_range(self) -> 'DateRange':
        def neutral_day(_date) -> int:
            if 2 != _date.month:
                is_leap_day = False
            else:
                is_leap_day = (29 == _date.day)
            # MINYEAR and the following year are not leap years
            return 28 if is_leap_day else _date.day

        low = date(MINYEAR, self.low.month, neutral_day(self.low))
        high = date(MINYEAR + 1 if self.spans_years() else MINYEAR, self.high.month, neutral_day(self.high))
        return DateRange(low, high)

    def with_month_offset(self, low_months: int, high_month: int) -> 'DateRange':
        pass

    def with_low_month_offset(self, months: int) -> 'DateRange':
        pass

    def with_high_month_offset(self, months: int) -> 'DateRange':
        pass

    @staticmethod
    def _days_in_month(year: int, month: int):
        return monthrange(year, month)[1]


class Location(NamedTuple):
    name: str
    alias: str
    longitude: str
    latitude: str
    tz: str

    def __eq__(self, other):
        """In weather data the location is identified by name and alias which allows this to work."""
        if isinstance(other, Location):
            return (self.name, other.name) == (self.alias, other.alias)
        raise NotImplemented

    def __hash__(self):
        """Since equality is base on name and alias this will work for a hash identifier."""
        return hash((self.name, self.alias))

    def __ne__(self, other):
        """Be explicit as to what not equal to means."""
        return not self.__eq__(other)

    def __repr__(self) -> str:
        return "(name='{}', alias={}, longitude={}, latitude={}, tz={})" \
            .format(self.name, self.alias, self.longitude, self.latitude, self.tz)

    def is_name(self, name: str, case_sensitive=False) -> bool:
        return name == self.name if case_sensitive else name.casefold() == self.name.casefold()

    def is_alias(self, alias: str, case_sensitive=False) -> bool:
        return alias == self.alias if case_sensitive else alias.casefold() == self.alias.casefold()

    def is_considered(self, value: str) -> bool:
        return self.is_name(value) or self.is_alias(value)

    class Field(Enum):
        NAME = "name"
        LONGITUDE = "longitude"
        LATITUDE = "latitude"
        ALIAS = "alias"
        TZ = "tz"

    def to_dict(self) -> dict:
        return {
            Location.Field.NAME.value: self.name,
            Location.Field.ALIAS.value: self.alias.casefold() if self.alias else self.alias,
            Location.Field.LONGITUDE.value: self.longitude,
            Location.Field.LATITUDE.value: self.latitude,
            Location.Field.TZ.value: self.tz
        }

    @staticmethod
    def from_dict(dictionary: dict) -> 'Location':
        def get_field(field_: Location.Field) -> str:
            data_ = dictionary.get(field_.value)
            if not data_:
                raise ValueError("The location {} is required.".format(field_.value))
            return str(data_)

        return Location(name=get_field(Location.Field.NAME),
                        alias=get_field(Location.Field.ALIAS).casefold(),
                        longitude=get_field(Location.Field.LONGITUDE),
                        latitude=get_field(Location.Field.LATITUDE),
                        tz=get_field(Location.Field.TZ))


class CityDB:
    class Record(NamedTuple):
        name: str
        state: str
        longitude: str
        latitude: str
        tz: str
        zips: str

        @staticmethod
        def from_dict(db_row: dict) -> 'CityDB.Record':
            return CityDB.Record(name=db_row["city"],
                                 state=db_row["state"],
                                 longitude=db_row["long"],
                                 latitude=db_row["lat"],
                                 tz=db_row["tz"],
                                 zips=db_row["zips"])

        def to_location(self) -> Location:
            return Location(name="{}, {}".format(self.name, self.state),
                            alias="{} {}".format(self.name, self.state).replace(" ", "_").casefold(),
                            longitude=self.longitude,
                            latitude=self.latitude,
                            tz=self.tz)

    def __init__(self):
        self._city_db: List[CityDB.Record] = []

        # PyCharm is having issues figuring out the import api
        # noinspection PyTypeChecker
        with open_binary_package(data, 'cities_db.csv.gz') as pkg_file:
            with GzipFile(mode="rb", fileobj=pkg_file) as gzip_file:
                for row in DictReader(TextIOWrapper(gzip_file, encoding="UTF-8")):
                    self._city_db.append(CityDB.Record.from_dict(row))

    def find(self, city: str = None, state: str = None, zip_code: str = None) -> List['CityDB.Record']:
        city_finder = re_compile(city.replace('*', '.*'), IGNORECASE) if city else None
        state_finder = re_compile(state, IGNORECASE) if state else None
        zip_code_finder = re_compile(zip_code.replace('*', '.*')) if zip_code else None

        matches = []
        for record in self._city_db:
            if city_finder and not city_finder.match(record.name):
                continue
            if state_finder and not state_finder.match(record.state):
                continue
            if zip_code_finder:
                matches = list(filter(zip_code_finder.match, record.zips.split()))
                continue
            matches.append(record)
        return matches


class WeatherProviderAPI:
    RECORDED = "recorded"
    ERROR = "error"
    API_CALLS_MADE = "api_calls_made"
    API_USAGE_LIMIT = 900
    API_REQUESTS_MADE_TODAY_HEADER = "X-Forecast-API-Calls"

    def __init__(self, key: str = None):
        self._key = key if key else get_setting("domain", "history_api_key")
        self._url = urljoin("https://api.darksky.net/forecast/", self._key) + "/"
        self._api_calls_made = 0

    @property
    def url(self) -> str:
        return self._url

    @property
    def key(self) -> str:
        return self._key

    def recorded(self, location: Location, when: datetime) -> dict:
        """
        The returned dictionary should always contain either RECORDED_KEY
        or ERROR_KEY. Optionally it can contain API_CALLS_MADE_KEY.
        """

        def mk_error(reason: str) -> dict:
            return {
                WeatherProviderAPI.ERROR: reason,
                WeatherProviderAPI.API_CALLS_MADE: self._api_calls_made
            }

        if self._api_calls_made > self.API_USAGE_LIMIT:
            return mk_error("You've made too many API requests to Dark Sky today...")

        url = urljoin(self.url, "{},{},{}".format(location.latitude, location.longitude, when.isoformat()))
        logging.debug("url: %s", url)
        try:
            response = requests.get(url, urlencode({"exclude": "currently,flags"}))
            if response.ok:
                api_calls = response.headers.get(self.API_REQUESTS_MADE_TODAY_HEADER.lower())
                logging.debug("api calls: %s", api_calls)
                if not api_calls:
                    print("Yikes... Didn't find {} header!!!".format(self.API_REQUESTS_MADE_TODAY_HEADER))
                    self._api_calls_made = self.API_USAGE_LIMIT + 1
                else:
                    self._api_calls_made = int(api_calls)
                return {
                    WeatherProviderAPI.RECORDED: response.json(),
                    WeatherProviderAPI.API_CALLS_MADE: self._api_calls_made
                }
            else:
                return mk_error("HTTP {}: {}".format(response.status_code, response.reason))
        except Exception as error:
            return mk_error(str(error))
