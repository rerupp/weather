import argparse
import json
import logging as log
import sys
from collections import OrderedDict
from datetime import date, timedelta
from enum import Enum
from os import linesep
from pathlib import Path
from typing import Callable, Dict, List, Optional

import pytz

from weather.domain import (
    DateRange,
    WeatherData, CsvDictWriter, Location, CityDB,
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


CommandExec = Callable[[argparse.Namespace, WeatherData], None]


class BaseCMD:

    def __init__(self, name: str):
        self._name: str = name

    @property
    def name(self) -> str:
        return self._name

    def add_to_parser(self, argument_parser):
        # noinspection PyProtectedMember
        log.error("%s needs to implement '%s(...)", self.__class__.__name__, sys._getframe(0).f_code.co_name)

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):
        # noinspection PyProtectedMember
        log.error("%s needs to implement '%s(...)", self.__class__.__name__, sys._getframe(0).f_code.co_name)


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

    @staticmethod
    def max_name_width(locations: List[Location]) -> int:
        return max(max((len(loc.name) for loc in locations)), len(" location "))


class ListLocationsCMD(BaseListCMD):

    def add_to_parser(self, argument_parser):
        super(ListLocationsCMD, self)._add_cmd(argument_parser, cmd_help="List weather locations.")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):
        locations = sorted(get_locations(weather_data, *options.locations), key=lambda l: l.name)
        if options.csv:
            with CsvDictWriter([f.value for f in Location.Field]).stdout() as write:
                for location in locations:
                    write(location.to_dict())
            return

        if options.json:
            location_list = [loc.to_dict() for loc in locations]
            print(json.dumps({"locations": location_list}, indent=2))
            return

        location_width = self.max_name_width(locations)
        alias_width = max(max((len(loc.alias) for loc in locations)), len(" Alias "))
        tz_width = max(max((len(loc.tz) for loc in locations)), len(" Timezone "))
        long_lat_width = 12
        header = "{:-^{lw}} {:-^{kw}} {:->{gw}}/{:-<{gw}} {:-^{tw}}"
        details = "{:<{lw}} {:<{kw}} {:>{gw}}/{:<{gw}} {:<{tw}}"
        print(header.format("Location", "Alias", "Longitude", "Latitude", "Timezone",
                            lw=location_width, gw=long_lat_width, kw=alias_width, tw=tz_width))
        for location in locations:
            print(details.format(location.name, location.alias, location.longitude, location.latitude, location.tz,
                                 lw=location_width, gw=long_lat_width, kw=alias_width, tw=tz_width))


class ListWeatherCMD(BaseListCMD):

    def add_to_parser(self, argument_parser):
        super(ListWeatherCMD, self)._add_cmd(argument_parser, cmd_help="List weather history.")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):
        locations = sorted(get_locations(weather_data, *options.locations), key=lambda l: l.name)
        histories: Dict[str, List[DateRange]] = {
            location.name: weather_data.history_date_ranges(location) for location in locations
        }

        if not (options.csv or options.json):
            def show_location_dates(loc, starting, ending):
                log.info("{:<{lw}} {}{}".format(loc.name if loc else "",
                                                starting if starting else "None found.",
                                                " to {}".format(ending) if ending else "",
                                                lw=location_width))

            location_width = self.max_name_width(locations)
            log.info("{:-^{lw}} {:-^24}".format("Location", "History Dates", lw=location_width))
            for location in locations:
                weather_ranges = histories.get(location.name)
                if not weather_ranges:
                    show_location_dates(location, None, None)
                else:
                    for i, (start, end) in enumerate(weather_ranges):
                        show_location_dates(location if i == 0 else None, start, end)
            return

        # create a dictionary that can be written as csv or json
        class Keys(str, Enum):
            name = "name"
            dates = "dates"
            start = "start_date"
            end = "end_date"

        history_dicts = []
        for location in locations:
            history_dicts.append({
                Keys.name: location.name,
                Keys.dates: [{Keys.start: str(h[0]), Keys.end: str(h[1])} for h in histories.get(location.name)]
            })

        if options.json:
            print(json.dumps({"history": history_dicts}, indent=2))

        elif options.csv:
            with CsvDictWriter([k for k in Keys if k != Keys.dates]).stdout() as dict_write:
                for history_dict in history_dicts:
                    location_dict = {Keys.name: history_dict[Keys.name]}
                    dates = history_dict[Keys.dates]
                    if dates:
                        for history_date in dates:
                            dict_write({**location_dict, **history_date})
                    else:
                        dict_write(location_dict)


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
        if options.al_cmd == "df":
            location = self.location_from_fields(options, weather_data)
        else:
            location = self.location_from_city(options, weather_data)
        if location:
            weather_data.add_location(location)
        for location in weather_data.locations():
            print(location)

    @staticmethod
    def location_from_fields(options: argparse.Namespace, weather_data: WeatherData) -> Optional[Location]:
        location = get_location(weather_data, options.name, options.alias)
        if location:
            log.error("A Location already exits (name='%s' alias='%s')....", location.name, location.alias)
        else:
            return Location(name=options.name,
                            longitude=options.long,
                            latitude=options.lat,
                            alias=options.alias,
                            tz=options.tz)

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
        cmd.add_argument("starting", help="The starting date for weather data (YYYY-MM-DD).")
        cmd.add_argument("ending", nargs="?", help="The ending data for weather data (default is starting date).")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):

        location = weather_data.get_location(options.location)
        if not location:
            log.error("Location '{}' was not found...".format(options.location))
            return

        one_day = timedelta(days=1)
        starting = date.fromisoformat(options.starting)
        ending = date.fromisoformat(options.ending) if options.ending else (starting + one_day)

        history_dates = weather_data.history_dates(location)
        existing_history = set(history_dates) if history_dates else None
        history_dates = []
        history_dates_exist = []
        while starting < ending:
            if existing_history and starting in existing_history:
                history_dates_exist.append(starting)
            else:
                history_dates.append(starting)
            starting += one_day

        def date_lines(dates: List[date]) -> List[str]:
            lines = []
            line = ""
            for d in dates:
                line += ", {}".format(d) if line else str(d)
                if 72 < len(line):
                    lines.append(line)
                    line = ""
            if line:
                lines.append(line)
            return lines

        if len(history_dates_exist):
            log.warning("History dates exists for:\n%s", linesep.join(date_lines(history_dates_exist)))
        log.debug("History dates:\n%s", linesep.join(date_lines(history_dates)))

        weather_data.add_history(location, history_dates, lambda hd: log.info("getting %s", hd))


class ReportWeatherHistoryCMD(BaseCMD):
    FMT_JSON = "json"
    FMT_CSV = "csv"
    DEFAULT_FILE = "stdout"

    def add_to_parser(self, argument_parser):
        cmd = argument_parser.add_parser(self.name, help="Generate weather history for a location.")
        format_group = cmd.add_mutually_exclusive_group()
        format_group.add_argument("-csv", dest="csv", action='store_true',
                                  help="Create a report containing CSV formatted output (default).")
        format_group.add_argument("-json", dest="json", action='store_true',
                                  help="Create a report containing JSON formatted output.")
        cmd.add_argument("--hourly", dest="hourly", default=False, action="store_true",
                         help="Generate weather history by hour (default by day).")
        cmd.add_argument("--file", dest="file", default=self.DEFAULT_FILE,
                         help="Where report content will be written (default={}).".format(self.DEFAULT_FILE))
        cmd.add_argument("location", help="The location where weather data is being collected.")
        cmd.add_argument("starting", help="The starting date for weather data (YYYY-MM-DD).")
        cmd.add_argument("ending", nargs="?", help="The ending data for weather data (default is starting date).")

    def execute(self, options: argparse.Namespace, weather_data: WeatherData):

        hourly_data = options.hourly
        location = weather_data.get_location(options.location)
        if not location:
            log.error("Location '{}' was not found...".format(options.location))
            return
        if not weather_data.history_exists(location):
            log.warning("'%s' does not have any weather history.", location.name)
            return

        output_path = None if options.file == self.DEFAULT_FILE else Path(options.file)
        output_type = self.FMT_JSON if options.json else self.FMT_CSV
        if output_path:
            if output_path.suffix != output_type:
                output_path = output_path.with_suffix("." + output_type)
            if output_path.exists() and not output_path.is_file():
                raise ValueError("CSV output '{}' is not a file...".format(output_path))

        starting = date.fromisoformat(options.starting)
        ending = date.fromisoformat(options.ending) if options.ending else None
        history_dates = weather_data.history_dates(location, starting, ending)

        tz = pytz.timezone(location.tz)
        date_fmt = "%Y-%m-%d %H:%M:%S"
        if hourly_data:
            data_converters = OrderedDict([
                (HourlyWeatherContent.TIME, lambda v: DataConverter.to_date(v, tz, fmt=date_fmt)),
                (HourlyWeatherContent.TEMPERATURE, lambda v: DataConverter.to_fahrenheit(v)),
                (HourlyWeatherContent.APPARENT_TEMPERATURE, lambda v: DataConverter.to_fahrenheit(v)),
                (HourlyWeatherContent.WIND_SPEED, lambda v: DataConverter.to_str(v)),
                (HourlyWeatherContent.WIND_GUST, lambda v: DataConverter.to_str(v)),
                (HourlyWeatherContent.WIND_BEARING, lambda v: DataConverter.wind_bearing(v)),
                (HourlyWeatherContent.CLOUD_COVER, lambda v: DataConverter.to_str(v))
            ])
            data_converter = GenericDataConverter[HourlyWeatherContent](data_converters)
        else:
            data_converters = OrderedDict([
                (DailyWeatherContent.TIME, lambda v: DataConverter.to_date(v, tz, fmt=date_fmt)),
                (DailyWeatherContent.TEMPERATURE_HIGH, lambda v: DataConverter.to_fahrenheit(v)),
                (DailyWeatherContent.TEMPERATURE_HIGH_TIME, lambda v: DataConverter.to_time(v, tz, fmt=date_fmt)),
                (DailyWeatherContent.TEMPERATURE_LOW, lambda v: DataConverter.to_fahrenheit(v)),
                (DailyWeatherContent.TEMPERATURE_LOW_TIME, lambda v: DataConverter.to_time(v, tz, fmt=date_fmt)),
                (DailyWeatherContent.TEMPERATURE_MAX, lambda v: DataConverter.to_fahrenheit(v)),
                (DailyWeatherContent.TEMPERATURE_MAX_TIME, lambda v: DataConverter.to_time(v, tz, fmt=date_fmt)),
                (DailyWeatherContent.TEMPERATURE_MIN, lambda v: DataConverter.to_fahrenheit(v)),
                (DailyWeatherContent.TEMPERATURE_MIN_TIME, lambda v: DataConverter.to_time(v, tz, fmt=date_fmt)),
                (DailyWeatherContent.WIND_SPEED, lambda v: DataConverter.to_str(v)),
                (DailyWeatherContent.WIND_GUST, lambda v: DataConverter.to_str(v)),
                (DailyWeatherContent.WIND_GUST_TIME, lambda v: DataConverter.to_time(v, tz, fmt=date_fmt)),
                (DailyWeatherContent.WIND_BEARING, lambda v: DataConverter.wind_bearing(v)),
                (DailyWeatherContent.CLOUD_COVER, lambda v: DataConverter.to_str(v))
            ])
            data_converter = GenericDataConverter[DailyWeatherContent](data_converters)
        selected_content = list(data_converters.keys())
        if output_type == self.FMT_CSV:
            csv_writer = CsvDictWriter([e.value for e in selected_content])
            with csv_writer.file_writer(output_path) if output_path else csv_writer.stdout() as csv_write:
                for histories in weather_data.get_history(location, history_dates, hourly_history=hourly_data):
                    for history in histories:
                        converted_data = data_converter.convert_contents(history, selected_content)
                        csv_write({e.value: converted_data[e] for e in selected_content})
            return

        histories = [h for h in weather_data.get_history(location, history_dates, hourly_history=hourly_data)]
        if output_path:
            with output_path.open("w") as fp:
                json.dump(histories, fp, indent=2)
        else:
            print(json.dumps(histories, indent=2))


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
