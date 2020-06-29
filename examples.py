# from datetime import datetime, timedelta, timezone
# from pytz import timezone
# import pytz
# import json
# import os
# from zipfile import ZipFile, ZIP_DEFLATED
# from pathlib import Path

# timezones = [
#     pytz.utc,
#     pytz.timezone("America/Phoenix"),
#     pytz.timezone("America/Denver")
# ]
# fmt = '%Y-%m-%d %H:%M:%S %Z%z'
#
# dt = datetime(2019, 12, 6, 15, 30, tzinfo=pytz.utc)
# print("datetime:")
# for tz in timezones:
#     print("  {}: {} ({})".format(tz, dt.astimezone(tz).strftime(fmt), dt.astimezone(tz).isoformat()))
#
# dt = datetime(2019, 12, 6, 15, 30)
# print("timezone")
# for tz in timezones:
#     print("  {}: {} ({})".format(tz, tz.localize(dt).strftime(fmt), tz.localize(dt).isoformat()))
#
# utc = timezone(timedelta(0))
# td = timedelta(0)
# print("utc TZ: default='{}' named='{}'".format(timezone(td), timezone(td, name="custom")))
#
# td = timedelta(seconds=float("-25200"))
# print("mst TZ: default='{}' named='{}'".format(timezone(td), timezone(td, name="custom")))
#
# unk = pytz.timezone("rick")
# print(unk if unk else "???")

# name = "weather_response.json"
# with open(name, "r") as fd:
#     darksky_json = json.load(fd)
# tz = pytz.timezone(darksky_json.get("timezone"))
# for hour in darksky_json.get("hourly").get("data"):
#     time = hour.get("time")
#     # dt = datetime.fromtimestamp(time, tz=pytz.utc)
#     dt = datetime.fromtimestamp(time)
#     print("{}: temperature={} wind={} wind bearing={}".format(tz.localize(dt).strftime("%H:%M"),
#                                                               hour.get("temperature"),
#                                                               hour.get("windSpeed"),
#                                                               hour.get("windBearing")))

# zipfile_name="example.zip"
# directory = "data"
# files = os.listdir(directory)
# if len(files):
#     with ZipFile(zipfile_name, mode="w", compression=ZIP_DEFLATED) as zipfile:
#         for file in files:
#             filename = os.path.join(directory, file)
#             print(os.path.join(directory, file))
#             zipfile.write(filename)
#
# with ZipFile(zipfile_name) as zipfile:
#     for info in zipfile.infolist():
#         print("{}: size={} compressed={} modified={}"
#               .format(info.filename, info.file_size, info.compress_size, info.date_time))
#         with zipfile.open(info) as fd:
#             contents = json.load(fd)
#             print(json.dumps(contents, indent=2))

# path = "data"
# only_files = [f for f in os.listdir(path) if os.path.isfile(os.path.join(path, f))]
# only_files = [f for f in os.listdir(path)]
# only_files = [f for f in Path(path).iterdir() if f.is_file()]
# only_files = [f for f in Path(path).iterdir() if f.is_file()]
# only_files = [f for f in Path(path).glob("**/*.json") if f.is_file()]
# for file in only_files:
#     print(file)

# import argparse
# parser = argparse.ArgumentParser()
# parser.add_argument("-f", "--file", dest="file", default="foo.dat", nargs=1,
#                           help="The name of a file (default 'foo.dat').")
# # group = parser.add_mutually_exclusive_group()
# # group.add_argument("-l", "--list", dest="list", action="store_true", default=False, help="List something.")
# # group.add_argument("-a", "--add", dest="add", default=False, action="store_true", help="Add something.")
# # group.add_argument("-d", "--delete", dest="delete", default=False, action="store_true", help="Delete something.")
# sub_parsers = parser.add_subparsers(dest='parser')
# sub_parser = sub_parsers.add_parser("list", help="List something")
# sub_parser.add_argument("type", choices=['text', 'csv', 'json'], default='text', nargs='?',
#                         help="The listing type (default text)")
# sub_parser = sub_parsers.add_parser("add", help="Add something")
# sub_parser.add_argument("name", help="the name") # , nargs=1)
# sub_parser.add_argument("known_as", help="known as") # , nargs=1)
# options = parser.parse_args()
# print(options)

# print(datetime.today().strftime("-%Y%m%d%H%M%S"))
# import weather.data as wd
# from weather import LocationsData
# wd.DataSource(Path("data"))
# wd.DataSource(Path("weather.zip"))
# wd.DataSource(Path("weather_data.zip"))
# wd.DataSource(Path("test/data"))
# try:
#     wd.DataSource(Path("weather.ini"))
# except wd.WeatherDataError as error:
#     print(error)
#
# import json
# from typing import BinaryIO
#
# ds = wd.DataSource("ds-test")
# entry = "foo/bar/test.json"
# if ds.exists(entry):
#     print("{} exists...".format(entry))
#
#
# def content_getter(entry_name):
#     print("getter asked for '{}'".format(entry_name))
#     return json.dumps({"test": "value 6"}, indent=2)
#
#
# def content_reader(entry_name: str, fd: BinaryIO):
#     print("reader receiving: '{}'".format(entry_name))
#     print(json.load(fd))
#
#
# ds.write(entry, content_getter)
# ds.read(entry, content_reader)
# print("exists??? " + str(ds.exists(entry)))
# locations = LocationsData.load(ds)
# locations.remove(locations.get("stgeorge"))
# print(locations.to_dict())
# LocationsData.save(ds, locations)

# ds = wd.DataSource("ds-test.zip")
# entry = "test.json"
# if ds.data_path_exists(entry):
#     print("{} exists...".format(entry))
# ds.write(entry, lambda fp: fp.write(json.dumps({"test": "4"}).encode("UTF-8")))
# ds.read(entry, lambda  fp: print(json.load(fp)))

# from typing import Tuple, Optional
# # def func(state: bool) -> Tuple[bool, Optional[str]]:
# #     return (state, None) if state else (state, "error/condition")
# # if func(True):
# #     print("True")
# # if not func(False)[0]:
# #     print("False")
# # print(func(False))
# import pathlib
#
# def recurse(dir_path, file_paths = None):
#     file_paths = file_paths if file_paths else []
#     dirs = []
#     for file_path in dir_path.iterdir():
#         if file_path.is_dir():
#             print("dir: {}".format(file_path))
#             dirs.append(file_path)
#         elif file_path.is_file():
#             print("file: {}".format(file_path))
#             file_paths.append(file_path)
#         else:
#             print("???: {}".format(file_path))
#     for dp in dirs:
#         recurse(dp, file_paths)
#     return file_paths

# for path in recurse(Path("weather")):
#     print(path)
#
# from weather import DataSource
# ds = DataSource("weather")
# for path in ds.data_paths():
#     print(path)
# from datetime import datetime
# dt = datetime.today().timetuple()
# print(dt)
# print(dt[:-3])
# from datetime import date, timedelta
# starting = date.fromisoformat("2018-01-01")
# dates = [starting + timedelta(days=x) for x in range(0, 20)]
# date_str = ""
# for n, d in enumerate(dates):
#     date_str += ", {}".format(d) if date_str else str(d)
#     if 6 == n % 7:
#         print(date_str)
#         date_str = ""
# if date_str:
#     print(date_str)
# from weather import LocationsProviderAPI
# api = LocationsProviderAPI()
# print(api.get("Lake Havasu City, AZ"))

# def foo(*args: str):
#     print(str(args))
#
# foo("one")
# foo("one", "two")
# foo()

# return
###############################################################################
# daily weather history runner
###############################################################################
# import random as rand
# from datetime import date, timedelta
# from tkinter import *
# from tkinter.ttk import *
# from typing import List
#
# from weather.gui import *
#
# if __name__ == "__main__":
#     root = Tk()
#     notebook_width, notebook_height = 640, 400
#     root.title = "Graph Example"
#     root.columnconfigure(0, weight=1)
#     root.rowconfigure(0, weight=1)
#
#     notebook = Notebook(master=root, width=notebook_width, height=notebook_height, padding=5)
#     notebook.grid(sticky=NSEW)
#     notebook.rowconfigure(0, weight=1)
#     notebook.columnconfigure(0, weight=1)
#
#     graph = DailyTemperatureGraph(notebook,
#                                   (date(2019, 10, 1), date(2020, 4, 30)),
#                                   title="Sample Daily Temperature Graph",
#                                   temperature_range=(30, 100))
#     notebook.add(graph, text="Daily Graph")
#
#     # create some sample data for plotting
#     def make_samples(start_date, end_date):
#         high, high_offsets = 75, list(range(-10, 0)) + list(range(0, 11))
#         low, low_offsets = 45, list(range(-5, 0)) + list(range(0, 10))
#         samples: List[DailyTemperature] = []
#         for sample in [start_date + timedelta(days=d) for d in range(0, (end_date - start_date).days + 1)]:
#             samples.append(DailyTemperature(sample, low + rand.choice(low_offsets), high + rand.choice(high_offsets)))
#         return samples
#
#
#     graph.plot(make_samples(date(2014, 10, 1), date(2015, 4, 30)), color="blue")
#     # graph.plot(make_samples(date(2015, 10, 1), date(2016, 4, 30)), color="orange")
#     # graph.plot(make_samples(date(2016, 10, 1), date(2017, 4, 30)), color="yellow")
#     graph.plot(make_samples(date(2017, 10, 1), date(2018, 4, 30)), color="green")
#     graph.plot(make_samples(date(2018, 10, 1), date(2019, 4, 30)), color="red")
#
#     Sizegrip(master=root).grid(row=1, column=1, sticky=(S, E))
#
#     root.mainloop()
# import csv
# import io
# import gzip
# import time
#
#
# class StopWatch:
#
#     _start: float = 0.0
#     _end: float = _start
#
#     def __init__(self, in_ms=False):
#         self._in_ms=in_ms
#
#     def start(self):
#         self._start = self._end = time.perf_counter()
#
#     def end(self):
#         self._end = time.perf_counter()
#
#     def elapsed(self):
#         diff = self._end - self._start
#         return round(diff * 1000) if self._in_ms else diff
#
#     def __str__(self):
#         return "{}{}".format(self.elapsed(), "ms" if self._in_ms else "s")
#
#
# stop_watch = StopWatch(True)
# stop_watch.start()
# city_db = []
# with open("us_cities.csv") as csv_file:
#     reader = csv.DictReader(csv_file)
#     for row in reader:
#         # print(row['city_ascii'], row['state_id'], row['lat'], row['lng'], row['timezone'], row['zips'])
#         city_db.append({
#             "city": row['city_ascii'],
#             "state": row['state_id'],
#             "long": row['lng'],
#             "lat": row['lat'],
#             "tz": row['timezone'],
#             "zips": row['zips']
#         })
# stop_watch.end()
# print("initial load {}".format(stop_watch))
#
# stop_watch.start()
# with gzip.GzipFile("cities_db.csv.gz", 'wb') as gzip_file:
#     writer = csv.DictWriter(io.TextIOWrapper(gzip_file, encoding='UTF-8', newline='\n'),
#                             fieldnames=["city", "state", "long", "lat", "tz", "zips"])
#     writer.writeheader()
#     for row in city_db:
#         writer.writerow(row)
# stop_watch.end()
# print("gzip write: {}".format(stop_watch))
#
# stop_watch.start()
# new_city_db = []
# with gzip.open("cities_db.csv.gz", "rb") as gzip_file:
#     reader = csv.DictReader(io.TextIOWrapper(gzip_file, encoding='UTF-8'))
#     for row in reader:
#         new_city_db.append(row)
# stop_watch.end()
# print("gzip read: {}".format(stop_watch))
# pass
# ###############################################################################
# # weather history select runner
# ###############################################################################
# from datetime import date, timedelta
# from tkinter import *
# from tkinter.ttk import *
#
# from weather import *
# from weather.gui import *
#
# if __name__ == "__main__":
#
#     location_date_ranges = [
#         (Location("one", "1", "1", "1", "tz"), DateRange(date(2019, 10, 1), date(2020, 4, 30))),
#         (Location("two", "2", "1", "1", "tz"), DateRange(date(2019, 10, 1), date(2020, 4, 30))),
#         (Location("three", "3", "1", "1", "tz"), DateRange(date(2020, 1, 1), date(2020, 3, 31))),
#         (Location("four", "4", "1", "1", "tz"), DateRange(date(2019, 10, 1), date(2019, 12, 31))),
#     ]
#     history_date_mappings = WeatherDomain._get_history_date_mapping(location_date_ranges)
#
#     def runner():
#         dialog = WeatherHistoryGraphDatesDialog(root, history_date_mappings)
#         if dialog.canceled:
#             print("cancelled...")
#             root.quit()
#
#     root = Tk()
#     Button(root, text="Run", command=runner).pack()
#     root.mainloop()
####################################################################################
# DateRange tester
####################################################################################
# from datetime import date, timedelta, MINYEAR
# from weather.domain_objects import DateRange
#
# try:
#     def get_date(doit) -> date:
#         if doit:
#             return date.today()
#     DateRange(get_date(False))
#     assert False, "expected a value error to be thrown"
# except ValueError as error:
#     print("caught expected {} '{}'".format(error.__class__.__name__, error))
#
# date_range = DateRange(date.today())
# assert date_range.low == date_range.high, "expected low and high dates to be equal"
#
# d1 = date.today()
# d2 = date.today() + timedelta(days=10)
# dr = DateRange(d1, d2)
# assert dr.low == d1, "unexpected low date"
# assert dr.high == d2, "unexpected high date"
#
# try:
#     DateRange(date.today() + timedelta(days=1), date.today())
#     assert False, "A value error should be raised if the high date is less than the low date"
# except ValueError as error:
#     print("Caught expected ValueError '{}'".format(error))
#
# assert DateRange(date.today()).total_days() == 0, "expected total days to be 0"
# assert DateRange(date.today(), date.today() + timedelta(days=10)).total_days() == 10, "expected total days to be 10"
#
# assert not DateRange(date.today()).spans_years(), "did not expected a DateRange of today to span years"
# assert DateRange(date(2020, 12, 31), date(2021, 1, 1)).spans_years(), "expected DateRange to span years"
#
# dates = [d for d in DateRange(date.today()).get_dates()]
# assert len(dates) == 1, "expected 1 date to be generated"
# dates = [d for d in DateRange(date.today(), date.today() + timedelta(days=2)).get_dates()]
# assert len(dates) == 3, "expected 3 dates to be generated"
#
# ndr = DateRange(date(2020, 2, 29)).as_neutral_date_range()
# assert ndr.low.day == 28, "expected neutral date range day to be 28"
# assert ndr.low.month == 2, "unexpected neutral_date month"
# assert ndr.low.year == MINYEAR, "unexpected neutral date year"
# assert ndr.low == ndr.high, "expected neutral date range low and high to be equal"
#
# ndr = DateRange(date(2020, 12, 1), date(2021, 1, 1)).as_neutral_date_range()
# assert ndr.low.day == 1, "unexpected low neutral date day"
# assert ndr.low.month == 12, "unexpected low neutral date month"
# assert ndr.low.year == MINYEAR, "unexpected low neutral date year"
# assert ndr.high.day == 1, "unexpected high neutral date day"
# assert ndr.high.month == 1, "unexpected high neutral date month"
# assert ndr.high.year == MINYEAR + 1, "unexpected high neutral date year"
#
# d1 = d2 = date.today()
# assert DateRange(d1) == DateRange(d2), "expected DateRange of today to equal"
# d2 += timedelta(days=1)
# assert DateRange(d1) != DateRange(d2), "did not expect DateRange to equal"
#
# d1 = d2 = date.today()
# dr = DateRange(d1)
# assert dr in DateRange(d2), "expected date range of today to be contained"
#
# d1 -= timedelta(days=1)
# d2 += timedelta(days=1)
# assert dr in DateRange(d1, d2), "expected date range to be contained"
# assert DateRange(d1, d2) not in dr, "did not expect date range to be contained"
# ###################################################################################
# # configuration
# ###################################################################################
# import weather.configuration as cfg
# domain = cfg.get_config("Domain")
# gui = cfg.get_config("GUI")
# print("domain: {} gui: {}".format(domain, gui))
#
# # make sure what comes back is a copy
# domain.clear()
# gui.clear()
# assert 0 < len(cfg.get_config("domain")), "Yikes... domain section is empty!"
# assert 0 < len(cfg.get_config("gui")), "Yikes... gui section is empty!"
#
# colors = cfg.get_config("GUI", "graph_colors")
# print("colors: {}".format(colors))
# colors.clear()
# assert 0 < len(cfg.get_config("GUI", "graph_Colors"))
#
# domain_cfg = cfg.get_config("Domain", "history_api_key", "weather_data_dir")
# print("configs: {}".format(domain_cfg))
#
# print("default api key: '{}'".format(cfg.get_default_config("domain", "history_api_key")))
# print("config api key: '{}'".format(cfg.get_config_value("domain", "history_api_key")))
#
# print("missing section: '{}'".format(cfg.get_config("missing")))
# print("missing config value: '{}'".format(cfg.get_config_value("missing", "history_api_key")))
# ####################################################################################
# # color chooser
# ####################################################################################
# from tkinter import *
# from tkinter.ttk import *
# from weather.gui.dialogs import ColorPicker
# from weather.configuration import Color
# from typing import Optional
#
# color: Optional[Color] = None
#
#
# def runner():
#     global color
#     dialog = ColorPicker(root, color)
#     if not dialog.canceled:
#         color = dialog.selected_color
#         print(color)
#
#
# root = Tk()
# Button(root, text="Run", command=runner).pack()
# root.mainloop()
# #######################################################################################
# # canvas with frame example
# #######################################################################################
# from tkinter import *
# from tkinter.ttk import *
#
# root = Tk()
# master = Frame(root)
# master.pack(fill=BOTH, expand=True)
#
# # create the sample
#
# title = Label(master, text="Canvas with frame example.")
#
# canvas = Canvas(master)
# inner_frame = Frame(canvas, relief=GROOVE)
# inner_frame.grid(sticky=NSEW, padx=5)
# inner_frame.columnconfigure(0, weight=1)
# inner_frame.rowconfigure(0, weight=1)
#
# Label(inner_frame, text="A Title on the canvas frame").grid(row=0, column=0, columnspan=2, padx=5, pady=5)
# Label(inner_frame, text="LHS frame text").grid(row=1, column=0, padx=5, pady=5, sticky=EW)
# Label(inner_frame, text="RHS frame text").grid(row=1, column=1, padx=5, pady=5, sticky=EW)
# inner_frame.update()
# canvas_frame = canvas.create_window((0, 0), window=inner_frame, anchor=NW)
#
# # x_scrollbar = Scrollbar(master, orient=HORIZONTAL)
# y_scrollbar = Scrollbar(master, orient=VERTICAL)
#
# # use pack for the overall sample
# y_scrollbar.pack(side=RIGHT, fill=Y)
# # x_scrollbar.pack(side=BOTTOM, fill=X)
# title.pack(side=TOP, fill=Y)
# canvas.pack(fill=BOTH, expand=True, padx=5)
#
# # bind the scroll bars
# # x_scrollbar.configure(command=canvas.xview)
# y_scrollbar.configure(command=canvas.yview)
# # canvas.configure(xscrollcommand=x_scrollbar.set, yscrollcommand=y_scrollbar.set)
# canvas.configure(yscrollcommand=y_scrollbar.set)
#
# # if you include the x scrollbar it will always be active so this trick solves that
# # canvas.update()
# # bbox = canvas.bbox(ALL)
# # canvas.configure(scrollregion=(bbox[0], bbox[1], bbox[2] - 1, bbox[3]))
#
#
# def canvas_frame_width(event):
#     canvas_width = event.width
#     canvas.itemconfigure(canvas_frame, width=canvas_width)
#     print("canvas width={}".format(canvas_width))
#
#
# canvas.bind("<Configure>", canvas_frame_width)
#
#
# # this should not be used with the x scrollbar
# def canvas_scroll_region(event):
#     bbox = canvas.bbox(ALL)
#     canvas.configure(scrollregion=canvas.bbox(ALL))
#     print("scrollregion={}".format(bbox))
#
#
# inner_frame.bind("<Configure>", canvas_scroll_region)
#
# root.mainloop()
# ####################################################################################
# # settings dialog
# ####################################################################################
# from tkinter import *
# from tkinter.ttk import *
# from weather.gui.dialogs import SettingsDialog
#
#
# def runner():
#     dialog = SettingsDialog(root)
#     if not dialog.canceled:
#         print(str(dialog))
#
#
# root = Tk()
# Button(root, text="Run", command=runner).pack()
# root.mainloop()
# ###############################################################################
# # weather settings
# ###############################################################################
# from enum import Enum
# from weather.configuration import get_setting, get_default_setting, get_settings, set_setting
# print("domain configuration: {}".format(get_settings("DOMAIN")))
# print("cli configuration: {}".format(get_settings("cLi")))
#
# EnumName = Enum('EnumName', [(k.casefold(), k.casefold()) for k in ["gui", "graph_colors", "history_api_key"]])
#
# colors = get_setting(EnumName.gui, EnumName.graph_colors)
# print("colors: {}".format(colors))
# colors.clear()
# print("config colors: {}".format(get_setting("gui", EnumName.graph_colors)))
#
# new_colors = ["orange", "yellow", "violet"]
# set_setting("gui", "graph_colors", new_colors)
# new_colors.clear()
# print("new colors: {}".format(get_setting("gui", EnumName.graph_colors)))
# set_setting("gui", "graph_colors", [])
# print("colors after delete: {}".format(get_setting("gui", "graph_colors")))
# set_setting("domain", EnumName.history_api_key, "The user key")
# print("current history_api_key: {}".format(get_setting("domain", EnumName.history_api_key)))
# print("default history_api_key: {}".format(get_default_setting("domain", EnumName.history_api_key)))
# print("config foobar: {}".format(get_setting("foo", "bar")))
# print("default foobar: {}".format(get_default_setting("foo", "bar")))
#
# base = ['blue2', 'dark green', 'DarkOrange1', 'dark violet', 'saddle brown']
# test = ['blue2', 'dark green', 'DarkOrange1', 'dark violet', 'saddle brown']
# assert base == test
# test = ['dark green', 'blue2', 'DarkOrange1', 'dark violet', 'saddle brown']
# assert base != test
# test = base[:1].copy()
# assert base != test
###############################################################################
# logging settings
###############################################################################
# from logging import WARNING, INFO, DEBUG
# from weather.configuration import get_logger, set_setting, set_module_logging_levels
# module_names = [
#     "weather.gui.gui_module_1",
#     "weather.gui.gui_module_2",
#     "weather.domain.domain_module",
#     "weather.cli"
# ]
# set_setting("gui", "logging_level", "info")
# set_setting("domain", "logging_level", "debug")
# set_setting("cli", "logging_level", "info")
# loggers = [get_logger(name) for name in module_names]
# assert loggers[0].isEnabledFor(INFO)
# assert loggers[1].isEnabledFor(INFO)
# assert loggers[2].isEnabledFor(DEBUG)
# assert loggers[3].isEnabledFor(INFO)
#
# set_setting("gui", "logging_level", "warning")
# set_setting("domain", "logging_level", "warning")
# set_setting("cli", "logging_level", "warning")
# set_module_logging_levels()
# assert not loggers[0].isEnabledFor(INFO)
# assert not loggers[1].isEnabledFor(INFO)
# assert not loggers[2].isEnabledFor(DEBUG)
# assert not loggers[2].isEnabledFor(INFO)
# assert not loggers[3].isEnabledFor(INFO)
# assert loggers[0].isEnabledFor(WARNING)
# assert loggers[1].isEnabledFor(WARNING)
# assert loggers[2].isEnabledFor(WARNING)
# assert loggers[3].isEnabledFor(WARNING)
from concurrent.futures import as_completed, ThreadPoolExecutor
from typing import List, Tuple
import orjson
from weather import StopWatch
from weather.domain import Location, WeatherData, WeatherHistory


class HistoryPerformance:

    def __init__(self):
        self.weather_data = WeatherData()

    def run(self, single_threaded, skip_load):
        load_control_blocks: List[Tuple[Location, WeatherHistory]] = []
        for location in self.weather_data.locations():
            # noinspection PyProtectedMember
            weather_history = self.weather_data._get_weather_history(location)
            load_control_blocks.append((location, weather_history))

        for p in range(2):
            print("{} pass {}threaded {}loading json".format(
                "2nd" if p else "1st",
                "single " if single_threaded else "multi-",
                "skip " if skip_load else ""
            ))
            read_stats: List[Tuple[Location, Tuple]] = []
            overall_stop_watch = StopWatch()
            if single_threaded:
                for location, weather_history in load_control_blocks:
                    read_stats.append((location, self.load_history(weather_history, skip_load)))
            else:
                with ThreadPoolExecutor(max_workers=20) as executor:
                    future_to_location = {
                        executor.submit(self.load_history, wh, skip_load): loc for loc, wh in load_control_blocks
                    }
                    for future in as_completed(future_to_location):
                        location = future_to_location[future]
                        if location:
                            try:
                                read_stats.append((location, future.result()))
                            except Exception as err:
                                print(f'"{location.name}" error: {str(err)}')
            overall_stop_watch.stop()
            print(f'History read took: {str(overall_stop_watch)}')
            for location, stats in read_stats:
                if stats:
                    stop_watch, histories = stats
                    loads_per_second = histories / stop_watch.elapsed() if stop_watch.elapsed() else float(histories)
                    print('"{}":(elapsed={},histories={},loads/sec={:,.1f})'.format(
                        location.name, stop_watch, histories, loads_per_second
                    ))

    @staticmethod
    def load_history(weather_history: WeatherHistory, skip_load: bool):
        if not weather_history:
            return

        history_count = 0
        stop_watch = StopWatch()
        with weather_history.reader() as history_reader:
            for history_date in weather_history.dates():
                history_count += 1

                data = history_reader(weather_history.make_pathname(history_date))
                if skip_load:
                    continue
                # full_history = json.loads(data)
                full_history = orjson.loads(data)
                # full_history = ujson.loads(data)

                hourly_history = full_history.get("hourly")
                assert hourly_history, "Did not find 'hourly' in weather history"

                daily_history = full_history.get("daily")
                assert daily_history, "Did not find 'daily' in weather history"
        stop_watch.stop()
        return stop_watch, history_count


if __name__ == "__main__":
    HistoryPerformance().run(single_threaded=True, skip_load=True)
    HistoryPerformance().run(single_threaded=True, skip_load=False)
    HistoryPerformance().run(single_threaded=False, skip_load=True)
    HistoryPerformance().run(single_threaded=False, skip_load=False)
