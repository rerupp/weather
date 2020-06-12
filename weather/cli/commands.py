import argparse
import json
import logging as log
from collections import OrderedDict, namedtuple
from datetime import date, timedelta
from decimal import Decimal, DecimalException
from enum import Enum
from io import StringIO
from pathlib import Path
from shutil import get_terminal_size
from typing import Callable, Dict, Generator, Iterable, List, Optional, TypeVar, Tuple

import pytz

from weather.domain import (
    DateRange,
    WeatherData, CsvDictWriter, Location, CityDB, WeatherHistoryProperties,
    DailyWeatherContent, HourlyWeatherContent, DataConverter, GenericDataConverter
)


def get_locations(weather_data: WeatherData, *location_names: str) -> List[Location]:
    if location_names:
        return [location for location in (weather_data.get_location(name) for name in location_names) if location]
    return [location for location in weather_data.locations()]


def get_location(weather_data: WeatherData, name: str, alias: str = None) -> Optional[Location]:
    location = weather_data.get_location(name)
    if not location and alias:
        location = weather_data.get_location(alias)
    return location


T = TypeVar('T')


def max_width(items: Iterable[T], min_width: int = 0, get: Callable[[T], str] = lambda t: t):
    return max(min_width, max((len(get(item)) for item in items), default=0))


def to_date(date_str: str) -> date:
    try:
        return date.fromisoformat(date_str)
    except (ValueError, TypeError):
        log.error("The date '{}' must be ISO format (YYYY-MM-DD).".format(date_str))


CommandExec = Callable[[argparse.Namespace, WeatherData], None]


class BaseCMD:

    def __init__(self, name: str):
        self._name: str = name

    @property
    def name(self) -> str:
        return self._name

    def add_to_parser(self, argument_parser):
        assert False, "{} needs to implement 'add_to_parser'!".format(self.__class__.__name__)

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):
        assert False, "{} needs to implement 'execute'!".format(self.__class__.__name__)


class BaseListCMD(BaseCMD):
    JSON = "json"
    CSV = "csv"
    TEXT = "text"

    def __init__(self, name: str):
        super().__init__(name)

    def _add_cmd(self, argument_parser, cmd_help: str):
        cmd = argument_parser.add_parser(self.name, help=cmd_help)
        list_group = cmd.add_mutually_exclusive_group()
        list_group.add_argument("-text", dest="text", action='store_true', default=False,
                                help="The output simple formatted text (default).")
        list_group.add_argument("-csv", dest="csv", action='store_true', default=False,
                                help="The output will be CSV formatted text.")
        list_group.add_argument("-json", dest="json", action='store_true', default=False,
                                help="The output will be JSON formatted text.")
        cmd.add_argument("locations", nargs=argparse.REMAINDER,
                         help="The locations that will be listed.")


class ListLocationsCMD(BaseListCMD):

    def add_to_parser(self, argument_parser):
        super(ListLocationsCMD, self)._add_cmd(argument_parser, cmd_help="List weather locations.")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):
        locations = sorted(get_locations(weather_data, *options.locations), key=lambda l: l.name)
        if not locations:
            return
        if options.csv:
            with CsvDictWriter([f.value for f in Location.Field]).stdout() as write:
                for location in locations:
                    write(location.to_dict())
            return

        if options.json:
            location_list = [loc.to_dict() for loc in locations]
            print(json.dumps({"locations": location_list}, indent=2))
            return

        header = "{:-^{lw}} {:-^{aw}} {:->{gw}}/{:-<{gw}} {:-^{tw}}"
        details = "{:<{lw}} {:<{aw}} {:>{gw}}/{:<{gw}} {:<{tw}}"
        column_widths = {
            "lw": max_width(locations, get=lambda loc: loc.name),
            "aw": max_width(locations, min_width=7, get=lambda loc: loc.alias),
            "gw": 12,
            "tw": max_width(locations, min_width=0, get=lambda loc: loc.tz)
        }
        print(header.format("Location", "Alias", "Longitude", "Latitude", "Timezone",
                            **column_widths))
        for location in locations:
            print(details.format(location.name, location.alias, location.longitude, location.latitude, location.tz,
                                 **column_widths))


class ListPropertiesCMD(BaseListCMD):

    def add_to_parser(self, argument_parser):
        super()._add_cmd(argument_parser, cmd_help="List weather data properties.")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):

        history_properties = weather_data.history_properties()
        if not history_properties:
            return
        history_properties.sort(key=lambda hp: hp[0].name)

        location_field_name = "location"
        property_field_names = WeatherHistoryProperties._fields

        def to_dict(_location: Location, _properties: WeatherHistoryProperties) -> dict:
            _dict = {location_field_name: _location.name}
            if _properties:
                for _field in property_field_names:
                    _dict[_field] = getattr(_properties, _field)
            return _dict

        if options.csv:
            with CsvDictWriter([location_field_name] + list(property_field_names)).stdout() as write:
                for location, properties in history_properties:
                    write(to_dict(location, properties))
            return

        if options.json:
            location_list = [to_dict(location, properties) for location, properties in history_properties]
            print(json.dumps({"locations": location_list}, indent=2))
            return

        def fmt(value: int, to_kib=True) -> str:
            if not value:
                return ""
            return "{: >,d}".format(value) if not to_kib or value < 1024 else "{: >,d} kiB".format(round(value / 1024))

        Field = namedtuple('Field', ['label', 'key', 'header', 'detail', 'width'])
        fields = (
            Field("Location", "lnw", "{:^{lnw}}", "{:<{lnw}}",
                  lambda: max_width(history_properties, 10, lambda hp: hp[0].name)),
            Field("Overall Size", "osw", "{:^{osw}}", "{:>{osw}}", lambda: len(fields[1].label)),
            Field("History Count", "hcw", "{:^{hcw}}", "{:>{hcw}}", lambda: len(fields[2].label)),
            Field("Raw History Size", "hsw", "{:^{hsw}}", "{:>{hsw}}", lambda: len(fields[3].label)),
            Field("Compressed Size", "csw", "{:^{csw}}", "{:>{csw}}", lambda: len(fields[4].label)),
        )
        header = " ".join([field.header for field in fields])
        field_sizes = {field.key: field.width() for field in fields}

        print(header.format(*(field.label for field in fields), **field_sizes))
        details = " ".join([field.detail for field in fields])
        print(details.format(*("-" * field.width() for field in fields), **field_sizes))
        for location, properties in history_properties:
            print(details.format(
                location.name,
                fmt(properties.size),
                fmt(properties.entries, to_kib=False),
                fmt(properties.entries_size),
                fmt(properties.compressed_size),
                **field_sizes)
            )
        print(details.format(*("=" * field.width() for field in fields), **field_sizes))
        print(details.format(
            "Totals",
            fmt(sum(p.size for _, p in history_properties)),
            fmt(sum(p.entries for _, p in history_properties), to_kib=False),
            fmt(sum(p.entries_size for _, p in history_properties)),
            fmt(sum(p.compressed_size for _, p in history_properties)),
            **field_sizes
        ))


class ListWeatherCMD(BaseListCMD):

    def add_to_parser(self, argument_parser):
        super(ListWeatherCMD, self)._add_cmd(argument_parser, cmd_help="List weather history.")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):
        locations = sorted(get_locations(weather_data, *options.locations), key=lambda l: l.name)
        histories: Dict[str, List[DateRange]] = {
            location.name: weather_data.history_date_ranges(location) for location in locations
        }

        if not (options.csv or options.json):
            field_widths = {
                "lw": max_width(locations, get=lambda l: l.name),
                "dw": len("YYYY-MM-DD to YYYY-MM-DD")
            }
            print("{:-^{lw}} {:-^{dw}}".format("Location", "History Dates", **field_widths))
            details = "{:<{lw}} {:<{dw}}"
            for location in locations:
                weather_ranges = histories.get(location.name)
                if not weather_ranges:
                    print(details.format(location.name, "None.", **field_widths))
                else:
                    for i, (start, end) in enumerate(weather_ranges):
                        print(details.format("" if i else location.name,
                                             "{} to {}".format(start, end) if start and end else str(start),
                                             **field_widths))
            return

        DictKeys = namedtuple('DictKeys', ["name", "dates", "start", "end"])

        def to_dict(_dict_keys: DictKeys) -> List[Dict]:
            return [
                {
                    _dict_keys.name: _location.name,
                    _dict_keys.dates: [
                        {
                            _dict_keys.start: str(_start),
                            _dict_keys.end: str(_end)
                        }
                        for _start, _end in histories.get(_location.name)
                    ]
                }
                for _location in locations
            ]

        if options.json:
            print(json.dumps({"history": to_dict(DictKeys("location", "dates", "start", "end"))}, indent=2))

        elif options.csv:
            fields = DictKeys("location", "dates", "start_date", "end_date")
            with CsvDictWriter([fields.name, fields.start, fields.end]).stdout() as dict_write:
                for history_dict in to_dict(fields):
                    location_dict = {fields.name: history_dict[fields.name]}
                    dates = history_dict[fields.dates]
                    if not dates:
                        dict_write(location_dict)
                    else:
                        for history_date in dates:
                            dict_write({**location_dict, **history_date})


class AddLocationCMD(BaseCMD):

    def add_to_parser(self, argument_parser):
        cmd = argument_parser.add_parser(self.name, help="Add a new Location.")
        sub_cmd = cmd.add_subparsers(title="Add", help="Add location by field names or city DB.", dest="al_cmd")

        fields = sub_cmd.add_parser("df", help="Add location using data field names.")
        fields.add_argument("--nm", required=True, metavar="LOC", dest="name", help="The Location name")
        fields.add_argument("--al", required=True, metavar="ALIAS", dest="alias", help="The Location alias")
        fields.add_argument("--lg", required=True, metavar="LONG", dest="long", help="The Location longitude")
        fields.add_argument("--lt", required=True, metavar="LAT", dest="lat", help="The Location latitude")
        fields.add_argument("--tz", required=True, metavar="TZ", dest="tz", help="The Location timezone")

        query = sub_cmd.add_parser("fc", help="Add location using the City DB")
        query.add_argument("--city", metavar="CITY", dest="city", help="The location city name.")
        query.add_argument("--state", metavar="STATE", dest="state", help="The location 2-digit state code.")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):
        location = None
        if options.al_cmd == "df":
            try:
                location = self.location_from_fields(options, weather_data)
            except ValueError as err:
                log.error(str(err), exc_info=True)
        else:
            location = self.location_from_city(options, weather_data)
        if location:
            weather_data.add_location(location)

    @staticmethod
    def location_from_fields(options: argparse.Namespace, weather_data: WeatherData) -> Optional[Location]:
        name = options.name
        if not name:
            raise ValueError("The location name is required.")

        def filter_alias(_alias) -> str:
            return ''.join(filter(lambda c: c.isalnum() or c in ("_", "-", " "), _alias)).replace(" ", "_")

        alias = filter_alias(options.alias)
        if not alias:
            alias = filter_alias(name)
        if get_location(weather_data, name, alias):
            raise ValueError("A location by that name or alias already exists.")

        tz = options.tz
        if not tz:
            raise ValueError("The location timezone is required.")
        try:
            pytz.timezone(tz)
        except pytz.UnknownTimeZoneError as _:
            raise ValueError("The location timezone is not valid.")

        def to_bounded_decimal(value, what: str, lower_bounds: int, upper_bounds: int) -> Decimal:
            try:
                decimal_value = Decimal(value)
                if decimal_value.is_nan() or lower_bounds >= decimal_value or decimal_value >= upper_bounds:
                    raise ValueError("{} must be between {} and {}.".format(what, lower_bounds, upper_bounds))
                return decimal_value
            except DecimalException as _:
                raise ValueError("The {} value '{}' is not valid.".format(what, value))

        longitude = to_bounded_decimal(options.long, "longitude", -180, 180)
        latitude = to_bounded_decimal(options.lat, "latitude", -90, 90)
        return Location(name=name,
                        longitude=str(longitude),
                        latitude=str(latitude),
                        alias=alias,
                        tz=tz)

    @staticmethod
    def location_from_city(options: argparse.Namespace, weather_data: WeatherData) -> Optional[Location]:
        location: Optional[Location] = None
        city_db = CityDB()
        if not (options.city or options.state):
            log.error("Either a city name or state needs to be provided...")
        else:
            cities = city_db.find(options.city, options.state)
            cities_len = len(cities)
            if not cities_len:
                log.error("The city was not found...")
            else:
                city_locations: List[Location] = []
                for city in cities:
                    city_location = city.to_location()
                    if not get_location(weather_data, city_location.name, city_location.alias):
                        city_locations.append(city_location)
                city_locations_len = len(city_locations)
                if 1 == city_locations_len:
                    location = city_locations[0]
                elif 1 < city_locations_len:
                    print("Multiple cities found.")
                    choice = "{: >2d}. {}" if 9 < city_locations_len else "{}. {}"
                    for idx, city_location in enumerate(city_locations):
                        print(choice.format(idx + 1, city_location.name))
                    selection = input("Enter city number or 'q' to cancel: ").strip()
                    if 'q' != selection:
                        city_idx = int(selection) - 1
                        if 0 <= city_idx < cities_len:
                            location = city_locations[city_idx]
                        else:
                            log.error("{} is not one of the city numbers...".format(city_idx))
        return location


class AddWeatherHistoryCMD(BaseCMD):

    def add_to_parser(self, argument_parser):
        cmd = argument_parser.add_parser(self.name, help="Add a weather location.")
        cmd.add_argument("location", help="The location where weather data is being collected.")
        cmd.add_argument("starting", help="The weather history starting date (YYYY-MM-DD).")
        cmd.add_argument("ending", nargs="?", help="The weather history ending date (default is starting date).")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):

        location = weather_data.get_location(options.location)
        if not location:
            log.error("Location '{}' was not found...".format(options.location))
            return

        one_day = timedelta(days=1)
        starting = to_date(options.starting)
        if not starting:
            return
        if not options.ending:
            ending = starting
        else:
            ending = to_date(options.ending)
            if not ending:
                return

        history_dates = weather_data.history_dates(location)
        existing_history = set(history_dates) if history_dates else set()
        history_dates = []
        history_dates_exist = []
        while starting <= ending:
            history_dates_exist.append(starting) if starting in existing_history else history_dates.append(starting)
            starting += one_day

        if history_dates_exist:
            with StringIO() as text:
                columns, _ = get_terminal_size(fallback=(80, 24))
                dates_per_line = int(columns / len(", YYYY-MM-DD"))
                for i, history_date in enumerate(history_dates_exist):
                    if i % dates_per_line:
                        text.write(", {}".format(history_date))
                    else:
                        text.write("\n{}".format(history_date))
                log.warning("History dates exists for:%s", text.getvalue())

        weather_data.add_history(location, history_dates, lambda hd: log.info("getting %s", hd))


class ReportWeatherHistoryCMD(BaseCMD):

    def add_to_parser(self, argument_parser):
        cmd = argument_parser.add_parser(self.name, help="Generate weather history for a location.")
        cmd.add_argument("--file", dest="file",
                         help="The file where report content will be written (default stdout).")

        cmd.add_argument_group().add_argument("--hourly", dest="hourly", default=False, action="store_true",
                                              help="Generate weather history by hour (default by day).")

        details_group = cmd.add_argument_group()
        details_group.add_argument("-t", "--temp", dest="temp", action='store_true',
                                   help="Include daily temperatures in report (default).")
        details_group.add_argument("-c", "--cnd", dest="cnd", action='store_true',
                                   help="Include conditions such as wind, uv, precipitation, etc. in report.")
        details_group.add_argument("-m", "--max", dest="max", action='store_true',
                                   help="Include min/max temperatures in daily report.")
        details_group.add_argument("-s", "--sum", dest="sum", action='store_true',
                                   help="Include a summary of the weather in report.")

        content_group = cmd.add_argument_group().add_mutually_exclusive_group()
        content_group.add_argument("-a", "--all", dest="all", action='store_true',
                                   help="Include all selected weather history (-t, -s, -c, etc.) in report.")
        content_group.add_argument("-r", "--raw", dest="raw", action='store_true',
                                   help="Create a JSON report containing the raw weather data.")

        output = cmd.add_argument_group()
        format_group = output.add_mutually_exclusive_group()
        format_group.add_argument("--text", dest="text", action='store_true',
                                  help="Create a report of formatted text (default).")
        format_group.add_argument("--csv", dest="csv", action='store_true',
                                  help="Create a report of CSV formatted output.")
        format_group.add_argument("--json", dest="json", action='store_true',
                                  help="Create a report of JSON formatted output.")

        cmd.add_argument("location", help="The location where weather data has been collected.")
        cmd.add_argument("starting", help="The starting date for weather data (YYYY-MM-DD).")
        cmd.add_argument("ending", nargs="?", help="The ending data for weather data (default is starting date).")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):

        location = weather_data.get_location(options.location)
        if not location:
            log.warning("Location '{}' was not found...".format(options.location))
            return
        if not weather_data.history_exists(location):
            log.warning("'{}' does not have any weather history.".format(location.name))
            return

        output_type = "json" if options.json else "csv" if options.csv else "txt"
        output_path = Path(options.file) if options.file else None
        if output_path:
            if output_path.suffix != output_type:
                output_path = output_path.with_suffix("." + output_type)
            if output_path.exists() and not output_path.is_file():
                raise ValueError("The {} destination, '{}', is not a file.".format(output_type, output_path))

        starting = to_date(options.starting)
        if not starting:
            return
        if not options.ending:
            ending = starting
        else:
            ending = to_date(options.ending)
            if not ending:
                return

        history_dates = weather_data.history_dates(location, starting, ending)

        # suck up all the histories
        histories = weather_data.get_history(location, history_dates, hourly_history=options.hourly)

        def to_json(_histories: Iterable) -> str:
            return json.dumps({
                "location": location.name,
                "type": "hourly_history" if options.hourly else "daily_history",
                "history": [_h for _h in _histories]
            }, indent=2)

        if options.raw:
            if output_path:
                output_path.write_text(to_json(histories))
            else:
                print(to_json(histories))
            return

        report_builder = self.ReportBuilder(location, options)

        if options.csv:
            csv_writer = CsvDictWriter(report_builder.data_converter.field_names())
            with csv_writer.file_writer(output_path) if output_path else csv_writer.stdout() as csv_write:
                for data in report_builder.data_generator(histories):
                    csv_write(data)
            return

        if options.json:
            history = to_json(report_builder.data_generator(histories))
            if output_path:
                output_path.write_text(history)
            else:
                print(history)
            return

        Field = namedtuple('Field', ['key', 'header', 'detail'])
        if report_builder.hourly:
            fields: Dict[Enum, Field] = {
                HourlyWeatherContent.TIME: Field("tw", "{:^{tw}}", "{:<{tw}}"),
                HourlyWeatherContent.TEMPERATURE: Field("tm", "{:^{tm}}", "{:<{tm}}"),
                HourlyWeatherContent.APPARENT_TEMPERATURE: Field("at", "{:^{at}}", "{:<{at}}"),
                HourlyWeatherContent.WIND_SPEED: Field("ws", "{:^{ws}}", "{:<{ws}}"),
                HourlyWeatherContent.WIND_GUST: Field("wg", "{:^{wg}}", "{:<{wg}}"),
                HourlyWeatherContent.WIND_BEARING: Field("wb", "{:^{wb}}", "{:<{wb}}"),
                HourlyWeatherContent.CLOUD_COVER: Field("cc", "{:^{cc}}", "{:<{cc}}"),
                HourlyWeatherContent.UV_INDEX: Field("uv", "{:^{uv}}", "{:<{uv}}"),
                HourlyWeatherContent.HUMIDITY: Field("hw", "{:^{hw}}", "{:<{hw}}"),
                HourlyWeatherContent.DEW_POINT: Field("dp", "{:^{dp}}", "{:<{dp}}"),
                HourlyWeatherContent.SUMMARY: Field("sw", "{:^{sw}}", "{:<{sw}})"),
            }
        else:
            fields: Dict[Enum, Field] = {
                DailyWeatherContent.TIME: Field("tw", "{:^{tw}}", "{:<{tw}}"),
                DailyWeatherContent.TEMPERATURE_HIGH: Field("th", "{:^{th}}", "{:^{th}}"),
                DailyWeatherContent.TEMPERATURE_HIGH_TIME: Field("tht", "{:^{tht}}", "{:^{tht}}"),
                DailyWeatherContent.TEMPERATURE_LOW: Field("tl", "{:^{tl}}", "{:^{tl}}"),
                DailyWeatherContent.TEMPERATURE_LOW_TIME: Field("tlt", "{:^{tlt}}", "{:^{tlt}}"),
                DailyWeatherContent.TEMPERATURE_MAX: Field("mx", "{:^{mx}}", "{:^{mx}}"),
                DailyWeatherContent.TEMPERATURE_MAX_TIME: Field("mxt", "{:^{mxt}}", "{:^{mxt}}"),
                DailyWeatherContent.TEMPERATURE_MIN: Field("mn", "{:^{mn}}", "{:^{mn}}"),
                DailyWeatherContent.TEMPERATURE_MIN_TIME: Field("mnt", "{:^{mnt}}", "{:^{mnt}}"),
                DailyWeatherContent.WIND_SPEED: Field("ws", "{:^{ws}}", "{:>{ws}}"),
                DailyWeatherContent.WIND_GUST: Field("wg", "{:^{wg}}", "{:>{wg}}"),
                DailyWeatherContent.WIND_GUST_TIME: Field("wgt", "{:^{wgt}}", "{:^{wgt}}"),
                DailyWeatherContent.WIND_BEARING: Field("wb", "{:^{wb}}", "{:^{wb}}"),
                DailyWeatherContent.CLOUD_COVER: Field("cc", "{:^{cc}}", "{:<{cc}}"),
                DailyWeatherContent.UV_INDEX: Field("uv", "{:^{uv}}", "{:>{uv}}"),
                DailyWeatherContent.UV_INDEX_TIME: Field("uvt", "{:^{uvt}}", "{:^{uvt}}"),
                DailyWeatherContent.SUMMARY: Field("sw", "{:<{sw}}", "{:<{sw}}"),
                DailyWeatherContent.HUMIDITY: Field("hw", "{:^{hw}}", "{:>{hw}}"),
                DailyWeatherContent.DEW_POINT: Field("dp", "{:^{dp}}", "{:<{dp}}"),
                DailyWeatherContent.SUNRISE_TIME: Field("sr", "{:^{sr}}", "{:^{sr}}"),
                DailyWeatherContent.SUNSET_TIME: Field("ss", "{:^{ss}}", "{:^{ss}}"),
                DailyWeatherContent.MOON_PHASE: Field("mp", "{:^{mp}}", "{:>{mp}}"),
            }

        selected_fields = report_builder.data_converter.keys()
        header = " ".join([fields[f].header for f in selected_fields])
        detail = " ".join([fields[f].detail for f in selected_fields])
        field_widths = {fields[f].key: report_builder.label_widths[f] for f in selected_fields}
        print(header.format(*(report_builder.labels[f][0] for f in selected_fields), **field_widths))
        print(header.format(*(report_builder.labels[f][1] for f in selected_fields), **field_widths))
        print(header.format(*("-" * field_widths[fields[f].key] for f in selected_fields), **field_widths))

        def safe_format(value):
            return "" if value is None else value

        for history in report_builder.data_generator(histories):
            print(detail.format(*(safe_format(history[f]) for f in selected_fields), **field_widths))

    class ReportBuilder:

        def __init__(self, location: Location, options: argparse.Namespace):
            self.all = options.all
            if self.all:
                self.temperatures = True
                self.max_temperatures = True
                self.summary = True
                self.conditions = True
            else:
                self.temperatures = options.temp
                self.max_temperatures = options.max
                self.summary = options.sum
                self.conditions = options.cnd
            if not (self.temperatures or self.summary or self.conditions or self.max_temperatures):
                self.temperatures = True
            self.hourly = options.hourly
            self.json = options.json
            self.csv = options.csv
            self.text = True if not (self.json or self.csv) else False

            tz = pytz.timezone(location.tz)

            date_fmt = "%Y-%m-%d" if self.text else "%Y-%m-%d %H:%M:%S"
            date_len = len(date.today().strftime(date_fmt))
            ts_fmt = "%H:%M" if self.text else date_fmt
            ts_len = len(date.today().strftime(ts_fmt))
            if self.hourly:
                data_converters: Dict[HourlyWeatherContent, Callable] = OrderedDict([
                    (HourlyWeatherContent.TIME, lambda v: DataConverter.to_date(v, tz, fmt=ts_fmt))
                ])
                if self.temperatures:
                    data_converters.update([
                        (HourlyWeatherContent.TEMPERATURE, lambda v: DataConverter.to_fahrenheit(v)),
                        (HourlyWeatherContent.APPARENT_TEMPERATURE, lambda v: DataConverter.to_fahrenheit(v)),
                    ])
                if self.conditions:
                    data_converters.update([
                        (HourlyWeatherContent.WIND_SPEED, lambda v: DataConverter.to_str(v)),
                        (HourlyWeatherContent.WIND_GUST, lambda v: DataConverter.to_str(v)),
                        (HourlyWeatherContent.WIND_BEARING, lambda v: DataConverter.wind_bearing(v)),
                        (HourlyWeatherContent.CLOUD_COVER, lambda v: DataConverter.to_str(v)),
                        (HourlyWeatherContent.UV_INDEX, lambda v: DataConverter.to_str(v))
                    ])
                if self.summary:
                    data_converters.update([
                        (HourlyWeatherContent.HUMIDITY, lambda v: DataConverter.to_str(v)),
                        (HourlyWeatherContent.DEW_POINT, lambda v: DataConverter.to_fahrenheit(v)),
                        (HourlyWeatherContent.SUMMARY, lambda v: DataConverter.to_str(v)),
                    ])
                # this should load from the configuration
                self.labels: Dict[Enum: Tuple[str, str]] = {
                    HourlyWeatherContent.TIME: ("", "Time"),
                    HourlyWeatherContent.TEMPERATURE: ("", "Temperature"),
                    HourlyWeatherContent.APPARENT_TEMPERATURE: ("Apparent", "Temperature"),
                    HourlyWeatherContent.WIND_SPEED: ("Wind", "Speed"),
                    HourlyWeatherContent.WIND_GUST: ("Wind", "Gust"),
                    HourlyWeatherContent.WIND_BEARING: ("Wind", "Bearing"),
                    HourlyWeatherContent.CLOUD_COVER: ("Cloud", "Cover"),
                    HourlyWeatherContent.UV_INDEX: ("UV", "Index"),
                    HourlyWeatherContent.HUMIDITY: ("", "Humidity"),
                    HourlyWeatherContent.DEW_POINT: ("Dew", "Point"),
                    HourlyWeatherContent.SUMMARY: ("", "Summary"),
                }

                def width(key: HourlyWeatherContent, min_width: int = 0) -> Tuple[Enum, int]:
                    return key, max_width(self.labels[key], min_width)
                self.label_widths: Dict[Enum: int] = dict([
                    width(HourlyWeatherContent.TIME, ts_len),
                    width(HourlyWeatherContent.TEMPERATURE),
                    width(HourlyWeatherContent.APPARENT_TEMPERATURE),
                    width(HourlyWeatherContent.WIND_SPEED, len("###.##")),
                    width(HourlyWeatherContent.WIND_GUST, len("###.##")),
                    width(HourlyWeatherContent.WIND_BEARING),
                    width(HourlyWeatherContent.CLOUD_COVER),
                    width(HourlyWeatherContent.UV_INDEX),
                    width(HourlyWeatherContent.HUMIDITY),
                    width(HourlyWeatherContent.DEW_POINT),
                    width(HourlyWeatherContent.SUMMARY, 25),
                ])
            else:
                data_converters: Dict[DailyWeatherContent, Callable] = OrderedDict([
                    (DailyWeatherContent.TIME, lambda v: DataConverter.to_date(v, tz, fmt=date_fmt))
                ])
                if self.temperatures:
                    data_converters.update([
                        (DailyWeatherContent.TEMPERATURE_HIGH, lambda v: DataConverter.to_fahrenheit(v)),
                        (DailyWeatherContent.TEMPERATURE_HIGH_TIME, lambda v: DataConverter.to_time(v, tz, fmt=ts_fmt)),
                        (DailyWeatherContent.TEMPERATURE_LOW, lambda v: DataConverter.to_fahrenheit(v)),
                        (DailyWeatherContent.TEMPERATURE_LOW_TIME, lambda v: DataConverter.to_time(v, tz, fmt=ts_fmt)),
                    ])
                if self.max_temperatures:
                    data_converters.update([
                        (DailyWeatherContent.TEMPERATURE_MAX, lambda v: DataConverter.to_fahrenheit(v)),
                        (DailyWeatherContent.TEMPERATURE_MAX_TIME, lambda v: DataConverter.to_time(v, tz, fmt=ts_fmt)),
                        (DailyWeatherContent.TEMPERATURE_MIN, lambda v: DataConverter.to_fahrenheit(v)),
                        (DailyWeatherContent.TEMPERATURE_MIN_TIME, lambda v: DataConverter.to_time(v, tz, fmt=ts_fmt))
                    ])
                if self.conditions:
                    data_converters.update([
                        (DailyWeatherContent.WIND_SPEED, lambda v: DataConverter.to_float(v, precision=1)),
                        (DailyWeatherContent.WIND_GUST, lambda v: DataConverter.to_float(v, precision=1)),
                        (DailyWeatherContent.WIND_GUST_TIME, lambda v: DataConverter.to_time(v, tz, fmt=ts_fmt)),
                        (DailyWeatherContent.WIND_BEARING, lambda v: DataConverter.wind_bearing(v)),
                        (DailyWeatherContent.CLOUD_COVER, lambda v: DataConverter.to_float(v, precision=2)),
                        (DailyWeatherContent.UV_INDEX, lambda v: DataConverter.to_str(v)),
                        (DailyWeatherContent.UV_INDEX_TIME, lambda v: DataConverter.to_time(v, tz, fmt=ts_fmt))
                    ])
                if self.summary:
                    data_converters.update([
                        (DailyWeatherContent.SUNRISE_TIME, lambda v: DataConverter.to_time(v, tz, fmt=ts_fmt)),
                        (DailyWeatherContent.SUNSET_TIME, lambda v: DataConverter.to_time(v, tz, fmt=ts_fmt)),
                        (DailyWeatherContent.MOON_PHASE, lambda v: DataConverter.to_float(v, precision=2)),
                        (DailyWeatherContent.HUMIDITY, lambda v: DataConverter.to_float(v, precision=2)),
                        (DailyWeatherContent.DEW_POINT, lambda v: DataConverter.to_fahrenheit(v)),
                        (DailyWeatherContent.SUMMARY, lambda v: DataConverter.to_str(v)),
                    ])
                # this should load from the configuration
                self.labels: Dict[Enum, Tuple[str, str]] = {
                    DailyWeatherContent.TIME: ("", "Date"),
                    DailyWeatherContent.TEMPERATURE_HIGH: ("High", "Temperature"),
                    DailyWeatherContent.TEMPERATURE_HIGH_TIME: ("High", "Temperature TOD"),
                    DailyWeatherContent.TEMPERATURE_LOW: ("Low", "Temperature"),
                    DailyWeatherContent.TEMPERATURE_LOW_TIME: ("Low", "Temperature TOD"),
                    DailyWeatherContent.TEMPERATURE_MAX: ("Maximum", "Temperature"),
                    DailyWeatherContent.TEMPERATURE_MAX_TIME: ("Maximum", "Temperature TOD"),
                    DailyWeatherContent.TEMPERATURE_MIN: ("Minimum", "Temperature"),
                    DailyWeatherContent.TEMPERATURE_MIN_TIME: ("Minimum", "Temperature TOD"),
                    DailyWeatherContent.WIND_SPEED: ("Wind", "Speed"),
                    DailyWeatherContent.WIND_GUST: ("Wind", "Gust"),
                    DailyWeatherContent.WIND_GUST_TIME: ("Wind", "Gust TOD"),
                    DailyWeatherContent.WIND_BEARING: ("Wind", "Bearing"),
                    DailyWeatherContent.CLOUD_COVER: ("Cloud", "Cover"),
                    DailyWeatherContent.UV_INDEX: ("UV", "Index"),
                    DailyWeatherContent.UV_INDEX_TIME: ("UV", "Index TOD"),
                    DailyWeatherContent.SUMMARY: ("", "Summary"),
                    DailyWeatherContent.HUMIDITY: ("", "Humidity"),
                    DailyWeatherContent.DEW_POINT: ("Dew", "Point"),
                    DailyWeatherContent.SUNRISE_TIME: ("", "Sunrise"),
                    DailyWeatherContent.SUNSET_TIME: ("", "Sunset"),
                    DailyWeatherContent.MOON_PHASE: ("Moon", "Phase"),
                }

                def width(key: DailyWeatherContent, min_width: int = 0) -> Tuple[Enum, int]:
                    return key, max_width(self.labels[key], min_width)
                self.label_widths: Dict[Enum, Tuple[str, str]] = dict([
                    width(DailyWeatherContent.TIME, date_len),
                    width(DailyWeatherContent.TEMPERATURE_HIGH),
                    width(DailyWeatherContent.TEMPERATURE_HIGH_TIME, ts_len),
                    width(DailyWeatherContent.TEMPERATURE_LOW),
                    width(DailyWeatherContent.TEMPERATURE_LOW_TIME, ts_len),
                    width(DailyWeatherContent.TEMPERATURE_MAX),
                    width(DailyWeatherContent.TEMPERATURE_MAX_TIME, ts_len),
                    width(DailyWeatherContent.TEMPERATURE_MIN),
                    width(DailyWeatherContent.TEMPERATURE_MIN_TIME, ts_len),
                    width(DailyWeatherContent.WIND_SPEED, len("###.##")),
                    width(DailyWeatherContent.WIND_GUST, len('###.##')),
                    width(DailyWeatherContent.WIND_GUST_TIME, ts_len),
                    width(DailyWeatherContent.WIND_BEARING),
                    width(DailyWeatherContent.CLOUD_COVER),
                    width(DailyWeatherContent.UV_INDEX),
                    width(DailyWeatherContent.UV_INDEX_TIME, ts_len),
                    width(DailyWeatherContent.SUMMARY, 25),
                    width(DailyWeatherContent.HUMIDITY),
                    width(DailyWeatherContent.DEW_POINT),
                    width(DailyWeatherContent.SUNRISE_TIME, ts_len),
                    width(DailyWeatherContent.SUNSET_TIME, ts_len),
                    width(DailyWeatherContent.MOON_PHASE),
                ])

            self.data_converter: GenericDataConverter = GenericDataConverter[Enum](data_converters)

        def data_generator(self, histories) -> Generator[dict, None, None]:
            selected_content = self.data_converter.field_names() if self.csv or self.json else self.data_converter.keys()
            return (self.data_converter.convert_contents(h, selected_content) for h in histories)


class RemoveWeatherDataCMD(BaseCMD):

    def add_to_parser(self, argument_parser):
        cmd = argument_parser.add_parser(self.name, help="Remove location and weather data.")
        cmd.add_argument("name", help="The weather data location name.")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):
        location = get_location(weather_data, options.name)
        if not location:
            log.error("Location '%s' does not exist...", options.name)
        elif not weather_data.remove_location(location):
            log.error("Weather data for '%s' was not completely removed...", options.name)
