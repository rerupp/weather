import os
from calendar import monthrange
from datetime import MINYEAR, date
from pathlib import Path
from tkinter import *
from tkinter import filedialog, messagebox
from tkinter.ttk import *
from typing import List, Tuple, Union, Dict, Any, Generator, Callable, Optional

import pytz

from weather.configuration import (
    get_setting, get_bool_setting, get_logger, set_module_logging_levels, init_logging, get_colors
)
from weather.domain import (
    WeatherData, Location, DailyWeatherContent, DataConverter, DateRange, GenericDataConverter, CityDB
)
from .widgets import (
    Coord, DailyTemperature, DailyTemperatureGraph, DailyWeatherWidget,
    LocationsWidget, NotebookWidget, StatusWidget
)
from .dialogs import (
    WeatherHistoryDialog, WeatherHistoryGraphDatesDialog, AddWeatherHistoryDialog,
    NewLocationDialog, WeatherDataPropertiesDialog,
    SettingsDialog
)

log = get_logger(__name__)

LocationDateRange = Tuple[Location, DateRange]
HistoryDateMapping = Tuple[Location, DateRange, List[bool]]
LocationViewSelection = Tuple[str, Union[Location, DateRange]]


class WeatherDomain:

    def __init__(self, root: Tk, notebook: NotebookWidget, status: StatusWidget, locations_view: LocationsWidget):
        self._weather_data: Optional[WeatherData] = None
        self._view_ids: Dict[str, Union[Location, DateRange]] = {}
        self._history_id_to_location_id: Dict[str, str] = {}
        self._root = root
        self._locations_view = locations_view
        self._notebook = notebook
        self._status = status

    @property
    def root(self) -> Tk:
        return self._root

    @property
    def weather_data(self) -> WeatherData:
        return self._weather_data

    def exit(self, prompt_to_exit=False):
        if prompt_to_exit:
            if messagebox.askyesno("Exit", "Exit Weather Data?") == NO:
                return
        self._root.quit()

    def get_id_value(self, view_id: str) -> Union[Location, DateRange]:
        return self._view_ids.get(view_id)

    def get_history_location(self, history_id: str) -> Location:
        location_id = self._history_id_to_location_id.get(history_id)
        if location_id:
            return self._view_ids.get(location_id)

    def set_menu(self, menu):
        self._root.configure(menu=menu)

    def get_selections(self) -> List[LocationViewSelection]:
        return [(selection, self.get_id_value(selection)) for selection in self._locations_view.get_selection()]

    def get_cursor_position(self) -> Coord:
        root = self._root
        return Coord(root.winfo_pointerx() - root.winfo_vrootx(), root.winfo_pointery() - root.winfo_vrooty())

    def change_weather_data_dir(self):
        if not self._weather_data:
            weather_data_dir = Path('.').absolute()
        elif self._weather_data.data_path().parent:
            weather_data_dir = self._weather_data.data_path().parent.absolute()
        else:
            weather_data_dir = self._weather_data.data_path().absolute()
        data_dir = filedialog.askdirectory(master=self._root,
                                           initialdir=weather_data_dir,
                                           title="Open Weather Data",
                                           mustexist=True)
        if data_dir:
            data_path = Path(data_dir)
            if not self._weather_data or data_path.absolute() != self._weather_data.data_path().absolute():
                if data_path.parent.absolute() == Path(".").absolute():
                    data_path = data_path.name
                if self._weather_data:
                    self._weather_data.close()
                log.debug("Creating weather data for {}".format(data_path))
                self._weather_data = WeatherData(data_path)
                self.load_locations()

    def load_locations(self, weather_data: Optional[WeatherData] = None):
        self._locations_view.clear()
        self._history_id_to_location_id.clear()
        self._view_ids.clear()

        if weather_data:
            self._weather_data = weather_data
        elif self._weather_data:
            weather_data = self._weather_data
        else:
            # this may happen if the app is started and there is no weather_data dir
            return

        self._status.set_weather_data_dir(weather_data)
        if weather_data:
            locations = sorted([loc for loc in weather_data.locations()], key=lambda l: l.name)
            log.debug("locations: {}".format([loc.name for loc in locations]))
            # noinspection PyTupleAssignmentBalance
            name, alias, longitude, latitude, tz, history = LocationsWidget.columns
            name_chars = name.min_chars
            alias_chars = alias.min_chars
            tz_chars = tz.min_chars
            for location in locations:
                name_chars = max(len(location.name), name_chars)
                alias_chars = max(len(location.alias), alias_chars)
                tz_chars = max(len(location.tz), tz_chars)
            self._locations_view.set_column_chars(name.id, name_chars)
            self._locations_view.set_column_chars(alias.id, alias_chars)
            self._locations_view.set_column_chars(tz.id, tz_chars)
            self._locations_view.set_column_chars(history.id, len("0000-00-00 to 0000-00-00"))

            for location in locations:
                location_id = self._locations_view.add_location(location)
                self._view_ids[location_id] = location
                for history_range in weather_data.history_date_ranges(location):
                    history_id = self._locations_view.add_history(location_id, history_range)
                    self._view_ids[history_id] = history_range
                    self._history_id_to_location_id[history_id] = location_id

    def add_location(self):
        if not self._weather_data:
            return
        city_db = CityDB()
        dialog = NewLocationDialog(self._root, city_db, [loc for loc in self._weather_data.locations()])
        if not dialog.canceled:
            self._weather_data.add_location(dialog.location)
            self.load_locations()

    def add_location_history(self):
        selections = self.get_selections()
        selection_count = len(selections)
        if 0 == selection_count:
            messagebox.showwarning(title="Add History", message="Select a location.")
            return

        if 1 == selection_count:
            # item_value = self.get_id_value(items[0])
            iid, item_value = selections[0]
            location = item_value if isinstance(item_value, Location) else self.get_history_location(iid)
        else:
            # make sure all the histories are from the same location
            location = self.get_history_location(selections[0][0])
            for item in selections[1:]:
                if self.get_history_location(item[0]) != location:
                    messagebox.showwarning(title="Add History", message="Select a single location.")
                    return

        add_history = AddWeatherHistoryDialog(self._root, title="Add History - {}".format(location.name))
        if add_history.canceled:
            return

        date_range = add_history.date_range
        history_dates = self._weather_data.history_dates(location)
        existing_history = set(history_dates) if history_dates else set()
        add_dates = []
        for add_date in date_range.get_dates():
            if add_date not in existing_history:
                add_dates.append(add_date)

        if 0 == len(add_dates):
            messagebox.showwarning(title="Add History", message="History already exists\nfor the selected dates...")
        else:
            progress = self._status.create_progress_widget("add history", len(add_dates))
            try:
                self._weather_data.add_history(location, add_dates, lambda a: progress.step())
            finally:
                progress.end()
            self.load_locations()

    def delete_location(self):
        selections = self.get_selections()
        selection_count = len(selections)
        if 0 == selection_count:
            messagebox.showwarning(title="Delete", message="Select a location.")
            return
        iid, item_value = selections[0]
        if not isinstance(item_value, Location):
            return
        if 1 < selection_count:
            messagebox.showwarning(title="Delete", message="Only 1 Location can be deleted at a time.")
            return
        if messagebox.askyesno(title="Delete", message="Delete {}?".format(item_value.name)) == YES:
            self._weather_data.remove_location(item_value)
            self.load_locations()

    def weather_data_properties(self):
        if not self._weather_data:
            return
        history_properties = self._weather_data.history_properties()
        if not history_properties:
            messagebox.showinfo(title="Weather Data properties", message="Weather Data is empty...")
        else:
            make_content = WeatherDataPropertiesDialog.Property.make
            contents: List[WeatherDataPropertiesDialog.Property] = []
            history_properties.sort(key=lambda p: p[0].name)
            for location_properties in history_properties:
                location, properties = location_properties
                if not properties:
                    content = make_content(location.name)
                else:
                    content = make_content(location.name,
                                           size=properties.size,
                                           entries=properties.entries,
                                           entries_size=properties.entries_size,
                                           compressed_size=properties.compressed_size)
                contents.append(content)
            WeatherDataPropertiesDialog(self._root, contents)

    DataConverters = Dict[DailyWeatherContent, Callable[[Any], Any]]
    DailyWeatherContentGenerator = Generator[Dict[DailyWeatherContent, Any], None, None]

    def report_history(self):
        selections = self.get_selections()
        if 1 != len(selections):
            raise RuntimeError("Yikes... The length of selections for report history is {}!".format(len(selections)))

        # get the history selection metrics
        iid, date_range = selections[0]
        history_selection = WeatherHistoryDialog(self._root, date_range, selected_content=[
            DailyWeatherContent.TIME,
            DailyWeatherContent.TEMPERATURE_HIGH,
            DailyWeatherContent.TEMPERATURE_HIGH_TIME,
            DailyWeatherContent.TEMPERATURE_LOW,
            DailyWeatherContent.TEMPERATURE_LOW_TIME
        ])
        if history_selection.canceled:
            return

        location = self.get_history_location(iid)
        date_range = history_selection.get_date_range()
        selected_content = history_selection.get_content_selection()

        # add the daily weather widget to the notebook
        widget = DailyWeatherWidget(self._notebook, history_selection.get_content_selection())
        start, end = date_range
        if end.year > start.year:
            tab_dates = "{} to {}".format(start.strftime("%b %Y"), end.strftime("%b %Y"))
        elif end.month > start.month:
            tab_dates = "{} to {}".format(start.strftime("%b"), end.strftime("%b %Y"))
        else:
            tab_dates = "{} to {}".format(start.strftime("%b %d"), end.strftime("%d %Y"))
        self._notebook.add_tab(widget, tab_name="{}\n{}".format(location.name, tab_dates))

        tz = pytz.timezone(location.tz)
        data_converters = {
            DailyWeatherContent.TIME: lambda v: DataConverter.to_date(v, tz, fmt="%b-%d-%Y"),
            DailyWeatherContent.TEMPERATURE_HIGH: lambda v: DataConverter.to_fahrenheit(v),
            DailyWeatherContent.TEMPERATURE_HIGH_TIME: lambda v: DataConverter.to_time(v, tz, fmt="%H:%M"),
            DailyWeatherContent.TEMPERATURE_LOW: lambda v: DataConverter.to_fahrenheit(v),
            DailyWeatherContent.TEMPERATURE_LOW_TIME: lambda v: DataConverter.to_time(v, tz, fmt="%H:%M"),
            DailyWeatherContent.TEMPERATURE_MAX: lambda v: DataConverter.to_fahrenheit(v),
            DailyWeatherContent.TEMPERATURE_MAX_TIME: lambda v: DataConverter.to_time(v, tz, fmt="%H:%M"),
            DailyWeatherContent.TEMPERATURE_MIN: lambda v: DataConverter.to_fahrenheit(v),
            DailyWeatherContent.TEMPERATURE_MIN_TIME: lambda v: DataConverter.to_time(v, tz, fmt="%H:%M"),
            DailyWeatherContent.WIND_SPEED: lambda v: DataConverter.to_str(v),
            DailyWeatherContent.WIND_GUST: lambda v: DataConverter.to_str(v),
            DailyWeatherContent.WIND_GUST_TIME: lambda v: DataConverter.to_time(v, tz, fmt="%H:%M"),
            DailyWeatherContent.WIND_BEARING: lambda v: DataConverter.wind_bearing(v),
            DailyWeatherContent.CLOUD_COVER: lambda v: DataConverter.to_str(v)
        }

        report_rows = []
        content_order = widget.content_order
        for history in self._get_location_histories(location, date_range, selected_content, data_converters):
            row = []
            for content in content_order:
                row.append(history[content])
            report_rows.append(row)

        widget.load(report_rows)

    def graph_history(self):

        selections = self.get_selections()
        selections_len = len(selections)
        if 0 == selections_len:
            return

        if 5 < selections_len:
            messagebox.showinfo(title="Graph", message="Current graph history only supports 5 selections.")
            return

        # to get here the controller should only have allowed a multiple selection of date ranges
        location_histories: List[LocationDateRange] = []
        for iid, selection in selections:
            assert isinstance(selection, DateRange), "Yikes... Selection for graph is not a DateRange!"
            location_histories.append((self.get_history_location(iid), selection))

        history_date_mappings = self._get_history_date_mapping(location_histories)
        if not history_date_mappings:
            return

        date_selection = WeatherHistoryGraphDatesDialog(self._root, history_date_mappings)
        if date_selection.canceled:
            return

        def start_end_months(_month_span: List[bool]) -> Tuple[int, int]:
            _start_month = _month_span.index(True)
            try:
                _end_month = _month_span.index(False, _start_month) - 1
            except ValueError:
                _end_month = len(_month_span) - 1
            return _start_month, _end_month

        graph_start_month, graph_end_month = start_end_months(date_selection.month_selections)

        def graph_date(_year: int, _month: int, _day: Callable[[int, int], int]) -> date:
            _year = (_year + 1) if 12 < _month else _year
            _month = (_month - 12) if 12 < _month else _month
            return date(_year, _month, _day(_year, _month))

        graph_date_range = DateRange(graph_date(MINYEAR, graph_start_month, lambda y, m: 1),
                                     graph_date(MINYEAR, graph_end_month, lambda y, m: monthrange(y, m)[1]))

        locations = [hdm[0] for hdm in history_date_mappings]
        multiple_locations = locations.count(locations[0]) != len(locations)
        if multiple_locations:
            title = "Daily Temperatures for Multiple Locations"
            tab_label = "History Graph for\nMultiple Locations"
        else:
            title = "Daily Temperatures for {}".format(location_histories[0][0].name)
            tab_label = "{}\nHistory Graph".format(location_histories[0][0].name)
        graph = DailyTemperatureGraph(self._notebook, graph_date_range, title=title)
        self._notebook.add_tab(graph, tab_label)

        # sanitize the colors jic...
        colors = get_setting("gui", "graph_colors")
        accepted_color_names = {c.name.casefold() for c in get_colors()}
        accepted_hex_color_names = {c.to_hex() for c in get_colors()}
        for idx, color in enumerate(colors):
            if color.casefold() not in accepted_color_names and color not in accepted_hex_color_names:
                log.warning("'%s' is not an accepted color. Review UI graph colors in File->Settings.", color)
                colors[idx] = "black"

        selected_content = [
            DailyWeatherContent.TIME,
            DailyWeatherContent.TEMPERATURE_LOW,
            DailyWeatherContent.TEMPERATURE_HIGH
        ]
        for location, location_date_range, location_month_slots in history_date_mappings:
            tz = pytz.timezone(location.tz)
            data_converters = {
                DailyWeatherContent.TIME: lambda v: DataConverter.to_binary_date(v, tz),
                DailyWeatherContent.TEMPERATURE_HIGH: lambda v: DataConverter.to_float(v),
                DailyWeatherContent.TEMPERATURE_LOW: lambda v: DataConverter.to_float(v)
            }

            location_start_month, location_end_month = start_end_months(location_month_slots)
            location_year = location_date_range.low.year
            if 12 < location_start_month and not location_date_range.spans_years():
                # adjust the location date range due to history being moved for date intersection
                location_year -= 1
            if location_start_month < graph_start_month:
                location_start_month = graph_start_month
            if location_end_month > graph_end_month:
                location_end_month = graph_end_month
            date_range = DateRange(graph_date(location_year, location_start_month, lambda y, m: 1),
                                   graph_date(location_year, location_end_month, lambda y, m: monthrange(y, m)[1]))

            daily_temperatures: List[DailyTemperature] = []
            for history in self._get_location_histories(location, date_range, selected_content, data_converters):
                daily_temperatures.append(DailyTemperature(history.get(DailyWeatherContent.TIME),
                                                           history.get(DailyWeatherContent.TEMPERATURE_LOW),
                                                           history.get(DailyWeatherContent.TEMPERATURE_HIGH)))
            sorted(daily_temperatures, key=lambda dt: dt.ts)

            color = colors[graph.plot_count % len(colors)]
            starting = daily_temperatures[0].ts
            ending = daily_temperatures[-1].ts
            if starting.year < ending.year:
                label = "{} to {}".format(starting.year, ending.year)
            else:
                label = str(starting.year)
            if multiple_locations:
                label = "{}\n{}".format(location.name, label)

            graph.plot(daily_temperatures, color=color, label=label)

    def settings(self):
        dialog = SettingsDialog(self._notebook)
        if not dialog.canceled:
            # assume the logging levels have changed
            set_module_logging_levels()

    def _get_location_histories(self,
                                location: Location,
                                date_range: DateRange,
                                selected_content: List[DailyWeatherContent],
                                data_converters: DataConverters) -> DailyWeatherContentGenerator:
        low_t, high_t = date_range
        history_dates = self._weather_data.history_dates(location, low_t, high_t)
        data_converter = GenericDataConverter[DailyWeatherContent](data_converters)
        progress = self._status.create_progress_widget("graph creation", maximum=len(history_dates))
        try:
            for history in self._weather_data.get_history(location, history_dates):
                progress.step()
                yield data_converter.convert_contents(history[0], selected_content)
        finally:
            progress.end()

    @staticmethod
    def _get_history_date_mapping(location_date_ranges: List[LocationDateRange]) -> List[HistoryDateMapping]:

        location_date_ranges_len = len(location_date_ranges)
        assert 0 < location_date_ranges_len, "Yikes... Location date ranges are emtpy!"

        # move the date range to a year neutral format
        neutral_date_ranges = [ldr[1].as_neutral_date_range() for ldr in location_date_ranges]

        # create the map location date months will go into to see where they might intersect
        month_slots: List[List[LocationDateRange]] = [[] for _ in range(25)]
        for idx, location_date_range in enumerate(location_date_ranges):
            date_range = neutral_date_ranges[idx]
            high_month = date_range.high.month
            if location_date_range[1].spans_years():
                high_month += 12
            for month in range(date_range.low.month, high_month + 1):
                month_slots[month].append(location_date_range)
        assert 0 == len(month_slots[0]), "Yikes... Month slot 0 has a value!"

        # make another pass across the months checking if 1-12 month dates should be moved to the next year
        for idx, month_slot in enumerate(month_slots):
            # only look at the first year of month slots
            if 12 < idx:
                break

            # check slots that have location date ranges and are not full
            if month_slot and len(month_slot) != location_date_ranges_len:
                month_slot_next_year = month_slots[idx + 12]
                if month_slot_next_year:
                    month_slot_next_year += [ldr for ldr in month_slot if not ldr[1].spans_years()]
                    month_slot[:] = [ldr for ldr in month_slot if ldr[1].spans_years()]

        # now search through the slots looking for intersecting month groups
        intersecting_months: List[Tuple[int, int]] = []
        start = end = 0
        for idx, month_slot in enumerate(month_slots):
            if month_slot:
                if start:
                    end = idx
                else:
                    start = end = idx
            elif start:
                intersecting_months.append((start, end))
                start = end = 0
        if start:
            intersecting_months.append((start, end))

        history_date_mappings: List[HistoryDateMapping] = []
        if 1 < len(intersecting_months):
            # todo: show the date groupings
            messagebox.showerror(title="Graph", message="History mapping selection is too complex...")
        else:
            # walk back through the intersecting months and get the location month slots for each location date range
            def eq(lhs: LocationDateRange, rhs: LocationDateRange) -> bool:
                return lhs[0].name == rhs[0].name and lhs[1].low == rhs[1].low and lhs[1].high == rhs[1].high

            start_month, end_month = intersecting_months[0]
            for location_date_range in location_date_ranges:
                history_month_slots = [False for _ in range(len(month_slots))]
                for slot in range(start_month, end_month + 1):
                    for slot_ldr in month_slots[slot]:
                        if eq(location_date_range, slot_ldr):
                            history_month_slots[slot] = True
                            break
                history_date_mappings.append((location_date_range[0], location_date_range[1], history_month_slots))
        return history_date_mappings


class WeatherController:

    def __init__(self, domain: WeatherDomain):
        self._domain = domain
        self._locations_menu = Menu(master=domain.root, tearoff=0)

        file_menu = Menu(self._locations_menu, tearoff=0)
        file_menu.add_command(label="Open", underline=0, command=self._domain.change_weather_data_dir)
        file_menu.add_command(label="Properties", underline=0, command=self._domain.weather_data_properties)
        file_menu.add_separator()
        file_menu.add_command(label="Settings", underline=0, command=self._domain.settings)
        file_menu.add_separator()
        file_menu.add_command(label="Exit", underline=1, command=self.ok_to_exit)
        self._locations_menu.add_cascade(label="File", underline=0, menu=file_menu)

        data_menu = Menu(self._locations_menu, tearoff=0)
        data_menu.add_command(label="Add Location", underline=4, command=self._domain.add_location)
        data_menu.add_command(label="Delete Location", underline=0, command=self._domain.delete_location)
        data_menu.add_separator()
        data_menu.add_command(label="Add History", underline=4, command=self._domain.add_location_history)
        self._locations_menu.add_cascade(label="Data", underline=0, menu=data_menu)

        def about():
            messagebox.showinfo(title="About", message="Weather Data GUI V0.5")

        help_menu = Menu(self._locations_menu, tearoff=0)
        help_menu.add_command(label="About...", underline=0, command=lambda: about())
        self._locations_menu.add_cascade(label="Help", underline=0, menu=help_menu)

        self._graph_menu = Menu(master=domain.root, tearoff=0)
        file_menu = Menu(self._graph_menu, tearoff=0)
        file_menu.add_command(label="Exit", underline=1, command=self.ok_to_exit)
        self._graph_menu.add_cascade(label="File", underline=0, menu=file_menu)

        self._report_menu = Menu(master=domain.root, tearoff=0)
        file_menu = Menu(self._report_menu, tearoff=0)
        file_menu.add_command(label="Exit", underline=1, command=self.ok_to_exit)
        self._report_menu.add_cascade(label="File", underline=0, menu=file_menu)

    def tab_change_event(self, event):
        widget_name = event.widget.select()
        widget = self._domain.root.nametowidget(widget_name)
        if isinstance(widget, DailyTemperatureGraph):
            menu = self._graph_menu
        elif isinstance(widget, DailyWeatherWidget):
            menu = self._report_menu
        else:
            menu = self._locations_menu
        self._domain.set_menu(menu)

    def ok_to_exit(self):
        self._domain.exit(prompt_to_exit=True)

    def right_click_action(self):
        menu = None
        selections = self._domain.get_selections()
        selection_len = len(selections)
        if 1 == selection_len:
            iid, value = selections[0]
            if isinstance(value, Location):
                menu = Menu(self._domain.root, tearoff=0)
                menu.add_command(label="Add history", command=self._domain.add_location_history)
                menu.add_command(label="Delete location", command=self._domain.delete_location)
            else:
                menu = Menu(self._domain.root, tearoff=0)
                menu.add_command(label="Report History", command=self._domain.report_history)
                menu.add_command(label="Graph History", command=self._domain.graph_history)
        elif 1 < selection_len:
            menu = Menu(self._domain.root, tearoff=0)
            menu.add_command(label="Graph History", command=self._domain.graph_history)

        if menu:
            try:
                coord = self._domain.get_cursor_position()
                menu.tk_popup(coord.x, coord.y, 0)
            finally:
                menu.grab_release()

    def location_view_select(self, event):

        # you're only concerned with multiple selections
        items = event.widget.selection()
        if 1 == len(items):
            return
        tree = event.widget

        # get the item that has just been selected
        tree_coords = Coord(tree.winfo_pointerx() - tree.winfo_rootx(),
                            tree.winfo_pointery() - tree.winfo_rooty())
        selected_item = tree.identify('item', *tree_coords)
        selected_value = self._domain.get_id_value(selected_item)

        # if a location has been selected, deselect everything else
        if isinstance(selected_value, Location):
            for item in items:
                if item != selected_item:
                    tree.selection_remove(item)
            return

        # get the location of the selected date range
        # location = self._domain.get_history_location(selected_item)

        # only support history selection from one location right now
        for item in items:
            value = self._domain.get_id_value(item)
            if isinstance(value, Location):
                tree.selection_remove(item)
            # elif location != self._domain.get_history_location(item):
            #     tree.selection_remove(item)


class WeatherApplication:

    def __init__(self, master, **kwargs):
        if os.name == 'nt' and get_bool_setting("gui", "windows_native_theme"):
            style = Style()
            print("theme names: {}, current theme: {}".format(style.theme_names(), style.theme_use()))
            theme = "winnative"
            Style().theme_use(theme)
            print("theme set to: {}".format(style.theme_use()))

        master.title("Weather Data")
        master.columnconfigure(0, weight=1)
        master.rowconfigure(0, weight=1)

        frame = Frame(master, **kwargs)
        frame.grid(sticky=NSEW)
        frame.columnconfigure(0, weight=1)
        frame.rowconfigure(0, weight=1)

        notebook = NotebookWidget(frame)
        notebook.grid(row=0, column=0, sticky=NSEW)

        status = StatusWidget(frame)
        status.grid(row=1, column=0, sticky=(S, E, W))

        locations_view = LocationsWidget(frame)
        notebook.add_tab(locations_view, "Locations")

        domain = WeatherDomain(master, notebook, status, locations_view)
        controller = WeatherController(domain)
        # domain.root.configure(menu=controller.locations_menu)

        notebook.bind_notebook_tab_changed(controller.tab_change_event)
        locations_view.on_event("<Button-3>", lambda e: controller.right_click_action())
        locations_view.on_event("<<TreeviewSelect>>", controller.location_view_select)

        data_path = Path(WeatherData.WEATHER_DATA_DIR)
        # todo: change so WeatherData is always instantiated
        domain.load_locations(WeatherData(data_path) if data_path.exists() and data_path.is_dir() else None)

        window_width = frame.winfo_reqwidth()
        window_height = frame.winfo_reqheight()
        x_position = int(master.winfo_screenwidth() / 3 - window_width / 2)
        y_position = int(master.winfo_screenheight() / 3 - window_height / 2)
        frame.master.geometry("+{}+{}".format(x_position, y_position))
        frame.update()
        frame.master.wm_minsize(frame.winfo_width(), frame.winfo_height())

        master.protocol("WM_DELETE_WINDOW", master.quit)

        self._frame = frame

    def execute(self):
        self._frame.mainloop()


def run_gui() -> None:
    init_logging()
    tk_root = Tk()
    WeatherApplication(tk_root).execute()
