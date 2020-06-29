import os
from datetime import date, timedelta
from enum import Enum
from tkinter import *
from tkinter import messagebox, LabelFrame as tk_LabelFrame, Button as tk_Button
from tkinter.font import nametofont, BOLD
from tkinter.simpledialog import Dialog
from tkinter.ttk import *
from typing import Tuple, List, Callable, NamedTuple, Optional, Set, Dict

from tkcalendar import DateEntry

from weather.configuration import (
    get_colors, Color,
    SettingName, SettingValue,
    get_setting, get_bool_setting, get_default_setting, get_settings, set_setting, save_settings,
    get_logger
)
from weather.domain import DateRange, DailyWeatherContent, CityDB, Location
from .gui_utils import month_name, make_font_builder, Coord, is_pathname_valid

log = get_logger(__name__)


class WeatherDialog(Dialog):

    def __init__(self, master, title=""):
        self._canceled = True
        super().__init__(master, title)

    @property
    def canceled(self):
        return self._canceled

    def apply(self):
        self._canceled = False


class AddWeatherHistoryDialog(WeatherDialog):

    def __init__(self, master, title="Add Weather History"):
        self._to_date = date.today().replace(day=1) - timedelta(days=1)
        self._from_date = self._to_date.replace(day=1)
        super().__init__(master, title=title)

    @property
    def date_range(self):
        return DateRange(self._from_date, self._to_date)

    def body(self, master: Frame) -> Widget:
        date_label = Label(master=master, text="History Date Selection")
        date_label.grid(column=0, row=0, sticky=W, padx=5, pady=5)
        master.columnconfigure(0, weight=1)
        master.columnconfigure(1, weight=1)

        def mk_date_entry(label: str, selected_date: date) -> DateEntry:
            frame = Frame(master=master)
            label = Label(master=frame, text=label)
            label.grid(column=0, row=0, padx=(5, 0), pady=5, sticky=E)
            date_entry = DateEntry(master=frame,
                                   year=selected_date.year,
                                   month=selected_date.month,
                                   day=selected_date.day,
                                   date_pattern="yyyy-mm-dd")
            date_entry.grid(column=1, row=0, padx=(0, 5), pady=5, sticky=E)
            date_entry.set_date(selected_date)
            return date_entry

        from_date_entry = mk_date_entry("Starting:", self._from_date)
        from_date_entry.configure(maxdate=self._to_date)
        from_date_entry.master.grid(row=1, column=0, sticky=E)

        to_date_entry = mk_date_entry("Ending:", self._to_date)
        to_date_entry.configure(mindate=self._from_date)
        to_date_entry.master.grid(row=2, column=0, sticky=E)

        def from_date_selected():
            previous_from_date = self._from_date
            self._from_date = from_date_entry.get_date()
            to_date_entry.configure(mindate=self._from_date)
            if 365 < (self._to_date - self._from_date).days:
                to_date_entry.set_date(self._to_date - (previous_from_date - self._from_date))

        from_date_entry.bind("<<DateEntrySelected>>", lambda e: from_date_selected())

        def to_date_selected():
            previous_to_date = self._to_date
            self._to_date = to_date_entry.get_date()
            from_date_entry.config(maxdate=self._to_date)
            if 365 < (self._to_date - self._from_date).days:
                from_date_entry.set_date(self._from_date + (self._to_date - previous_to_date))

        to_date_entry.bind("<<DateEntrySelected>>", lambda e: to_date_selected())

        return from_date_entry

    def validate(self) -> bool:
        if 365 < (self._to_date - self._from_date).days:
            messagebox.showinfo(title="Weather History", message="History selection cannot exceed 1 year...")
            return False

        return True


class WeatherHistoryGraphDatesDialog(WeatherDialog):
    date_selections: List[IntVar]

    def __init__(self, master, history_date_mappings: List[Tuple[Location, DateRange, List[bool]]]):

        self.history_date_mappings = history_date_mappings

        # only deal with a 2 year span regardless the size of the history data mapping
        self.month_selections: List[bool] = [False] * 25

        # find the month range in the date mappings
        start_month = len(self.month_selections)
        end_month = 1
        for history_date_mapping in history_date_mappings:
            for idx, month in enumerate(history_date_mapping[2]):
                if month:
                    start_month = min(start_month, idx)
                    end_month = max(end_month, idx)
        self._start_month = start_month
        self._end_month = end_month

        super().__init__(master, title="Weather History Date Selection")

    def body(self, master):

        # row metrics
        header_rows = 3
        trailer_rows = 1
        grid_rows = len(self.history_date_mappings) + header_rows + trailer_rows

        # column metrics
        date_columns = self._end_month - self._start_month + 1
        date_starting_column = 2
        grid_columns = date_columns + date_starting_column

        # initialize the matrix used to hold frames
        grid_frames: List[List[Optional[Widget]]] = []
        for frames_row in range(grid_rows):
            # noinspection PyUnusedLocal
            grid_frames.append([None for y in range(grid_columns)])

        master_frame = Frame(master)
        master_frame.grid(padx=5, pady=5, sticky=NSEW)

        # the Dialog title
        style = Style()
        title_font = nametofont(style.lookup("TLabel", "font")).copy()
        title_font.configure(size=title_font.cget('size') + 1)
        style.configure('Title.TLabel', font=title_font)

        def add_title(_row: int, _text: str, _pady: Tuple[int, int]):
            _title_frame = Frame(master_frame)
            _title_frame.grid(row=_row, column=0, sticky=EW, columnspan=grid_columns)
            Label(_title_frame, text=_text, style='Title.TLabel').pack(pady=_pady)

        row = 0
        add_title(row, "Select the graph date range by clicking", _pady=(10, 0))
        row += 1
        add_title(row, "the date columns or check boxes.", _pady=(0, 10))

        def make_frame(_row: int, _column: int, _relief: str) -> Frame:
            _frame = Frame(master_frame, relief=_relief, borderwidth=1)
            _frame.grid(row=_row, column=_column, sticky=EW)
            _frame.grid_columnconfigure(index=_column, weight=1)
            _frame.grid_rowconfigure(index=_row, weight=1)
            grid_frames[_row][_column] = _frame
            return _frame

        def add_text(_text: str, _row: int, _column: int, _relief: str, _anchor: str = CENTER) -> Widget:
            _label = Label(make_frame(_row, _column, _relief), text=_text, anchor=_anchor)
            _label.pack(padx=5, pady=5)
            return _label

        def left_click(_event):
            # both the label and frame are tied to the left click event
            _frame = _event.widget if isinstance(_event.widget, Frame) else _event.widget.master

            # get the column you're in
            _grid_info = _frame.grid_info()
            _column = _grid_info['column']

            # the ttk check button does not have deselect, flash, select or toggle attributes
            _date_selector = self.date_selections[_column - date_starting_column]
            _date_selector.set(0 if _date_selector.get() else 1)

            # now fire the the check button command
            _check_button = check_buttons[_column - date_starting_column]
            month_selections_command(_check_button)

        month_selected = SUNKEN
        month_not_selected = RAISED

        def month_selections_command(_check_button: Checkbutton):
            _grid_info = _check_button.master.grid_info()
            _column = _grid_info['column']
            _button_state = self.date_selections[_column - date_starting_column].get()
            _relief = month_selected if _button_state else month_not_selected
            _state = NORMAL if _button_state else DISABLED

            # change the state of the date label
            _frame = grid_frames[header_rows - 1][_column]
            for _label in _frame.children.values():
                _label.config(state=_state)

            # now change the state of the date indicators
            for _row in range(len(self.history_date_mappings)):
                _row += header_rows
                _frame = grid_frames[_row][_column]
                _frame.config(relief=_relief)
                for _label in _frame.children.values():
                    _label.config(state=_state)

        # column headers
        header_font = nametofont(style.lookup("TLabel", "font")).copy()
        header_font.configure(weight=BOLD)
        style.configure('Header.TLabel', font=header_font)
        row += 1
        column_text = ["Locations", "Timeline"] + [month_name(m) for m in range(self._start_month, self._end_month + 1)]
        for column, text in enumerate(column_text):
            label = add_text(text, row, column, _relief=FLAT)
            label.configure(style='Header.TLabel')
            if date_starting_column <= column:
                label.bind("<Button-1>", left_click)

        # the location date selection rows
        def date_range_label(_date_range: DateRange) -> str:
            if _date_range.spans_years():
                _timeline = "{}-{}".format(_date_range.low.year, _date_range.high.year)
            else:
                _timeline = str(_date_range.low.year)
            return _timeline

        for location, date_range, month_slots in self.history_date_mappings:
            row += 1
            column_text = [location.name, date_range_label(date_range)]
            column_text += ["X" if month_slots[m] else "" for m in range(self._start_month, self._end_month + 1)]
            for column, text in enumerate(column_text):
                relief = GROOVE if 2 > column else month_selected
                label = add_text(text, row, column, _relief=relief)
                if date_starting_column <= column:
                    label.bind("<Button-1>", left_click)
                    grid_frames[row][column].bind("<Button-1>", left_click)

        # the check buttons row used to turn on/off the selection of a date column
        def add_checkbutton(_row: int, _column: int) -> Checkbutton:
            _check_button_var = self.date_selections[_column - date_starting_column]
            _check_button_var.set(1)
            _check_button = Checkbutton(make_frame(_row, _column, _relief=FLAT), variable=_check_button_var)
            _check_button.pack()
            _check_button.config(command=lambda: month_selections_command(_check_button))
            return _check_button

        row += 1
        check_buttons: List[Checkbutton] = []
        # noinspection PyUnusedLocal
        self.date_selections = [IntVar() for i in range(date_columns)]
        for column in range(date_starting_column, grid_columns):
            check_buttons.append(add_checkbutton(row, column))

    def validate(self):

        def show_error(_message):
            messagebox.showerror(title="Graph Dates Error", message=_message)

        # convert the list of IntVar to int
        selections: List[int] = [s.get() for s in self.date_selections]
        selections_len = len(selections)

        # find the start of the month selection
        selection_start = -1
        for idx in range(selections_len):
            if selections[idx]:
                selection_start = idx
                break
        if 0 > selection_start:
            show_error("At least 1 month must be selected.")
            return

        # find the end of the month selection
        selection_end = selection_start
        for idx in range(selections_len - 1, selection_start, -1):
            if selections[idx]:
                selection_end = idx
                break

        # make sure there isn't a gap between the start and end
        if (selection_end - selection_start) > 1:
            for idx in range(selection_start, selection_end + 1):
                if not selections[idx]:
                    show_error("The month selection must be contiguous and not contain a gap.")
                    return

        # now walk the locations to make sure the all have at least one month selected
        for history_date_mapping in self.history_date_mappings:
            month_slots = history_date_mapping[2][self._start_month: self._end_month + 1]
            if not month_slots[selection_start: selection_end + 1].count(True):
                show_error("{} does not have data to match current selection.".format(history_date_mapping[0].name))
                return

        return True

    def apply(self):
        selections: List[int] = [s.get() for s in self.date_selections]
        self.month_selections[self._start_month: self._end_month] = selections
        super().apply()


class WeatherHistoryDialog(WeatherDialog):

    def __init__(self,
                 master,
                 initial_from_to_dates: Tuple[date, date],
                 selected_content: List[DailyWeatherContent],
                 show_content_selection=True):
        self._from_date, self._to_date = initial_from_to_dates
        self._selected_content = set(selected_content) if len(selected_content) else set()
        self._content_selectors: List[Tuple[DailyWeatherContent, IntVar]] = []
        self._show_content_selection = show_content_selection
        super().__init__(master, title="Daily Weather History")

    def body(self, master: Frame) -> Widget:

        # date selection
        date_frame = Frame(master=master, relief=GROOVE, borderwidth=1, padding=5)
        date_frame.grid(padx=5, pady=5)

        date_label = Label(master=date_frame, text="History Dates")
        date_label.grid(sticky=W, padx=5, pady=5)
        master.columnconfigure(0, weight=1)
        master.columnconfigure(1, weight=1)

        def mk_date_entry(label: str, selected_date: date) -> DateEntry:
            frame = Frame(master=date_frame)
            frame.grid(sticky=E)
            Label(master=frame, text=label).grid(padx=(5, 0), pady=5, sticky=E)
            date_entry = DateEntry(master=frame,
                                   year=selected_date.year,
                                   month=selected_date.month,
                                   day=selected_date.day,
                                   date_pattern="yyyy-mm-dd",
                                   mindate=self._from_date,
                                   maxdate=self._to_date)
            date_entry.grid(column=1, row=0, padx=(0, 5), pady=5, sticky=E)
            return date_entry

        from_date_entry = mk_date_entry("Starts:", self._from_date)
        to_date_entry = mk_date_entry("Ends:", self._to_date - timedelta(days=1))

        def from_date_selected():
            self._from_date = from_date_entry.get_date()
            to_date_entry.configure(mindate=self._from_date)

        from_date_entry.bind("<<DateEntrySelected>>", lambda e: from_date_selected())

        def ending_date_selected():
            self._to_date = to_date_entry.get_date()
            from_date_entry.configure(maxdate=self._to_date)

        to_date_entry.bind("<<DateEntrySelected>>", lambda e: ending_date_selected())

        # content selection
        content_frame = Frame(master=master, relief=GROOVE, borderwidth=1, padding=5)
        if self._show_content_selection:
            content_frame.grid(padx=5, pady=5)

        def mk_selector(parent, content: DailyWeatherContent, text: str) -> Checkbutton:
            button_state = IntVar()
            button_state.set(1 if content in self._selected_content else 0)
            button = Checkbutton(parent, text=text, variable=button_state)
            button.grid(sticky=W)
            self._content_selectors.append((content, button_state))
            return button

        Label(master=content_frame, text="Contents Selection").grid(sticky=W, padx=5, pady=5)

        temperature_frame = Frame(content_frame, relief=GROOVE, borderwidth=1, padding=5)
        mk_selector(temperature_frame, DailyWeatherContent.TEMPERATURE_HIGH, "Daytime High")
        mk_selector(temperature_frame, DailyWeatherContent.TEMPERATURE_HIGH_TIME, "Daytime High TOD")
        mk_selector(temperature_frame, DailyWeatherContent.TEMPERATURE_LOW, "Overnight Low")
        mk_selector(temperature_frame, DailyWeatherContent.TEMPERATURE_LOW_TIME, "Overnight Low TOD")
        Label(temperature_frame, text="").grid(sticky=(W, E))
        mk_selector(temperature_frame, DailyWeatherContent.TEMPERATURE_MAX, "Daily High")
        mk_selector(temperature_frame, DailyWeatherContent.TEMPERATURE_MAX_TIME, "Daily High TOD")
        mk_selector(temperature_frame, DailyWeatherContent.TEMPERATURE_MIN, "Daily Low")
        mk_selector(temperature_frame, DailyWeatherContent.TEMPERATURE_MIN_TIME, "Daily Low TOD")

        other_frame = Frame(content_frame, relief=GROOVE, borderwidth=1, padding=5)
        mk_selector(other_frame, DailyWeatherContent.WIND_SPEED, "Wind Speed")
        mk_selector(other_frame, DailyWeatherContent.WIND_GUST, "Wind Gust Speed")
        mk_selector(other_frame, DailyWeatherContent.WIND_GUST_TIME, "Wind Gust Speed TOD")
        mk_selector(other_frame, DailyWeatherContent.WIND_BEARING, "Wind Bearing")
        Label(other_frame, text="").grid(sticky=(W, E))
        mk_selector(other_frame, DailyWeatherContent.CLOUD_COVER, "Cloud Cover")

        temperature_frame.grid(column=0, row=1, sticky=(N, W), padx=5, pady=5)
        other_frame.grid(column=1, row=1, sticky=(N, E), padx=5, pady=5)

        return from_date_entry

    def get_date_range(self) -> DateRange:
        return DateRange(self._from_date, self._to_date)

    def get_content_selection(self) -> List[DailyWeatherContent]:
        content_selection = [DailyWeatherContent.TIME]
        for content, value in self._content_selectors:
            if value.get():
                content_selection.append(content)
        return content_selection


class FindCityDialog(WeatherDialog):

    def __init__(self, master, city_db: CityDB, city_location_exists: Callable[[CityDB.Record], bool]):

        self._matching_cities: Optional[List[CityDB.Record]] = None
        self._city_selection: int = -1
        self._city_selector: Optional[Listbox] = None
        self._city_db = city_db

        # add the search fields
        self._city_name = StringVar()
        self._state_name = StringVar()
        self._search_results = StringVar()

        self._city_location_exists = city_location_exists

        super().__init__(master, title="Find City")

    @property
    def city(self) -> CityDB.Record:
        if 0 <= self._city_selection:
            return self._matching_cities[self._city_selection]

    def body(self, master: Frame) -> Widget:

        # create the search frame
        search_frame = Frame(master=master, padding=5)
        search_frame.grid(row=0, padx=5, pady=5)

        label_options = {"sticky": E, "padx": (5, 2), "pady": 5}
        entry_options = {"sticky": W, "padx": (0, 5), "pady": 5}

        Label(master=search_frame, text="City Name:").grid(row=0, column=0, **label_options)
        city_width = len("this is a really long city name")
        city = Entry(search_frame, width=city_width, textvariable=self._city_name)
        city.grid(row=0, column=1, **entry_options)

        def state_selector_key_pressed(event):
            # allow the state to be cleared
            if event.keysym == 'BackSpace' or event.keysym == 'Delete':
                self._state_name.set("")

            elif event.keysym == event.char:
                # a letter or digit was pressed
                matched_states = [s for s in states if event.char.upper() == s[0]]
                if 0 < len(matched_states):
                    current_selection = event.widget.current()
                    if -1 == current_selection:
                        matched_state = matched_states[0]
                    else:
                        selected_state = states[current_selection]
                        if selected_state not in matched_states:
                            matched_state = matched_states[0]
                        elif selected_state == matched_states[-1]:
                            matched_state = matched_states[0]
                        else:
                            matched_state = states[current_selection + 1]
                    self._state_name.set(matched_state)

        Label(master=search_frame, text="State:").grid(row=0, column=2, **label_options)
        states = sorted(["AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "DC", "FL",
                         "GA", "HI", "ID", "IL", "IN", "IA", "KS", "LA", "ME", "MD",
                         "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
                         "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
                         "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY"])
        state_selector = Combobox(master=search_frame,
                                  width=5,
                                  textvariable=self._state_name,
                                  height=10,
                                  values=states)
        state_selector.grid(row=0, column=3, **entry_options)
        state_selector.bind('<Any-Key>', state_selector_key_pressed)
        # prevent any characters from making it into the entry field
        state_validator = self.register(lambda: False)
        state_selector.configure(validate="key", validatecommand=state_validator)

        Button(master=search_frame,
               text="Find City",
               command=self._city_search,
               default=ACTIVE).grid(row=2, columnspan=4, pady=5)

        results_frame = Frame(master, relief=GROOVE, borderwidth=1, padding=5)
        results_frame.grid(row=1, padx=5, pady=5)

        Label(master=results_frame, text="Matching Cities").grid(row=0, pady=5)

        scroll_frame = Frame(results_frame)
        scroll_frame.grid(row=1, pady=5)

        scrollbar = Scrollbar(scroll_frame, orient=VERTICAL)
        self._city_selector = Listbox(master=scroll_frame,
                                      height=5,
                                      width=city_width,
                                      yscrollcommand=scrollbar.set,
                                      listvariable=self._search_results,
                                      selectmode=SINGLE)
        self._city_selector.grid(row=0, column=0, pady=5)
        scrollbar.configure(command=self._city_selector.yview)
        scrollbar.grid(row=0, column=1, sticky=(N, S, E))

        self.bind("<Return>", lambda e: self._city_search())
        self.bind("<Escape>", self.cancel)
        self._city_selector.bind("<Double-Button-1>", lambda e: self.ok())

        return city

    def validate(self):
        current_selection = self._city_selector.curselection()
        if len(current_selection):
            self._city_selection = current_selection[0]
            return True
        messagebox.showinfo(title="Find", message="A city was not selected.")

    def buttonbox(self):
        box = Frame(self)
        Button(box, text="OK", width=10, command=self.ok).pack(side=LEFT, padx=5, pady=5)
        Button(box, text="Cancel", width=10, command=self.cancel).pack(side=LEFT, padx=5, pady=5)
        box.pack()

    def _city_search(self):
        city = self._city_name.get()
        state = self._state_name.get()
        if not city and not state:
            return

        self._matching_cities = []
        for city in self._city_db.find(city.strip() if city else None, state.strip() if state else None):
            if not self._city_location_exists(city):
                self._matching_cities.append(city)
        if len(self._matching_cities):
            self._matching_cities.sort(key=lambda m: (m.name, m.state))
            self._search_results.set(["{}, {}".format(m.name, m.state) for m in self._matching_cities])


class NewLocationDialog(WeatherDialog):

    def __init__(self, master, city_db: CityDB, locations: List[Location]):
        self._city_db = city_db
        self._locations = locations
        self._location: Optional[Location] = None

        # the location fields
        self._location_name = StringVar()
        self._location_alias = StringVar()
        self._location_longitude = StringVar()
        self._location_latitude = StringVar()
        self._location_tz = StringVar()
        super().__init__(master, title="Add Location")

    def body(self, master: Frame) -> Widget:

        label_options = {"sticky": E, "padx": (5, 2), "pady": 5}
        entry_options = {"sticky": W, "padx": (0, 5), "pady": 5}

        def mk_entry(row: int, label: str, entry_variable: StringVar, entry_len: int) -> Entry:
            Label(master=master, text=label).grid(row=row, column=0, **label_options)
            entry = Entry(master, width=entry_len, textvariable=entry_variable)
            entry.grid(row=row, column=1, **entry_options)
            return entry

        name = mk_entry(0, "Name:", self._location_name, 40)
        mk_entry(1, "Alias:", self._location_alias, 40)
        long_entry = mk_entry(2, "Longitude:", self._location_longitude, 20)
        lat_entry = mk_entry(3, "Latitude:", self._location_latitude, 20)
        mk_entry(4, "Timezone:", self._location_tz, 20)

        def number_only(action, text):
            if '1' == action:
                for c in text:
                    if not (c.isdigit() or '.' == c or '-' == c or '+' == c):
                        return False
            return True

        number_validator = self.register(number_only)
        long_entry.configure(validate="key", validatecommand=(number_validator, '%d', '%S'))
        lat_entry.configure(validate="key", validatecommand=(number_validator, '%d', '%S'))

        Button(master=master,
               text="Fill out using City information",
               command=self._city_search).grid(row=5, columnspan=2, pady=5)

        return name

    def validate(self):
        def validation_error(reason: str):
            messagebox.showwarning(title="Add Error", message=reason)

        if 0 == len(self._location_name.get()):
            validation_error("Location name required!")
        elif 0 == len(self._location_alias.get()):
            validation_error("Location alias required!")
        elif 0 == len(self._location_longitude.get()):
            validation_error("Location longitude required!")
        elif 0 == len(self._location_latitude.get()):
            validation_error("Location latitude required!")
        elif 0 == len(self._location_tz.get()):
            validation_error("Location timezone required!")
        else:
            name = self._location_name.get()
            alias = self._location_alias.get()
            if self._location_exists(name, alias):
                validation_error("'{}'/{} already exists!".format(name, alias))
            else:
                return True

    def apply(self):
        super().apply()
        self._location = Location(name=self._location_name.get(),
                                  alias=self._location_alias.get(),
                                  longitude=self._location_longitude.get(),
                                  latitude=self._location_latitude.get(),
                                  tz=self._location_tz.get())

    @property
    def location(self):
        return self._location

    def _location_exists(self, name: str, alias: str) -> bool:
        for location in self._locations:
            if location.is_considered(name) or location.is_considered(alias):
                return True

    def _city_search(self):
        def city_location_exists(city_: CityDB.Record):
            location = city_.to_location()
            return self._location_exists(location.name, location.alias)

        city_finder = FindCityDialog(self, self._city_db, city_location_exists)
        if not city_finder.canceled:
            city = city_finder.city
            self._location_name.set("{}, {}".format(city.name, city.state))
            self._location_alias.set("{} {}".format(city.name, city.state).replace(" ", "_").casefold())
            self._location_longitude.set(city.longitude)
            self._location_latitude.set(city.latitude)
            self._location_tz.set(city.tz)


class WeatherDataPropertiesDialog(WeatherDialog):
    class Property(NamedTuple):
        name: str
        size: int
        entries: int
        entries_size: int
        compressed_size: int

        @staticmethod
        def make(name: str,
                 size: int = 0,
                 entries: int = 0,
                 entries_size: int = 0,
                 compressed_size: int = 0) -> 'WeatherDataPropertiesDialog.Property':
            """
            There are inspection warnings in PyCharm concerning what is being passed in as
            the first argument of the constructor. This method makes sure there are no issues.
            """
            return WeatherDataPropertiesDialog.Property._make([name, size, entries, entries_size, compressed_size])

    class Column(NamedTuple):
        id: str
        text: str
        heading_anchor: str
        anchor: str
        stretch: str

    columns: Tuple[Column] = (
        Column("#0", "Name", CENTER, W, YES),
        Column("size", "Overall Size", E, E, NO),
        Column("entries", "History Count", E, E, NO),
        Column("entries_size", "History Size", E, E, NO),
        Column("compressed_size", "Compressed Size", E, E, NO)
    )

    def __init__(self, master, properties: List[Property]):
        self._properties = properties
        super().__init__(master, title="Weather Data Properties")

    def body(self, master):

        master.pack(fill=BOTH, expand=Y)

        scrollbar = Scrollbar(master, orient=VERTICAL)
        scrollbar.pack(fill=Y, side=RIGHT, expand=FALSE)

        ids = [cid.id for cid in self.columns[1:]]
        tree = Treeview(master, columns=ids, selectmode="none", yscrollcommand=scrollbar.set)
        tree.pack(side=LEFT, fill=BOTH, expand=Y)

        scrollbar.config(command=tree.yview)

        def measure(text: str) -> int:
            return font.measure("X" * (len(text)))

        font = nametofont(Style().lookup(tree.winfo_class(), "font"))
        name_width = measure(self.columns[0].text)
        for prop in self._properties:
            name_width = max(name_width, measure(prop.name))

        # get the column sizes
        for column in self.columns:
            tree.heading(column.id, text=column.text, anchor=column.heading_anchor)
            width = name_width if "#0" == column.id else measure(column.text)
            tree.column(column.id, minwidth=width, width=width, anchor=column.anchor, stretch=column.stretch)

        def number_format(value: int) -> str:
            if value < 1000:
                return "{: >3d}".format(value)
            if value < 1024 * 1000:
                return "{: >3d}kb".format(round(value / 1024))
            return "{: >,d}MB".format(round(value / 1024 / 1024))

        formatted_values: List[Tuple[str, Tuple[str, str, str, str]]] = []
        for prop in self._properties:
            if 0 == prop.entries:
                formatted_values.append((prop.name, ("", "", "", "")))
            else:
                formatted_values.append((prop.name, (
                    number_format(prop.size),
                    "{: >8,d}".format(prop.entries),
                    number_format(prop.entries_size),
                    number_format(prop.compressed_size)
                )))

        for location, values in formatted_values:
            tree.insert("", "end", text=location, values=values)

    def buttonbox(self):
        box = Frame(self)
        Button(box, text="OK", width=10, command=self.ok).pack(side=LEFT, padx=5, pady=5)
        self.bind("<Return>", self.ok)
        self.bind("<Escape>", self.cancel)
        box.pack()


class ColorPicker(WeatherDialog):

    def __init__(self, master, selected_color: Color = None):
        self._selected_color = selected_color
        super().__init__(master, title="Color Picker")

    @property
    def selected_color(self):
        return self._selected_color

    def body(self, master):

        # setup the color picker window
        master.pack(expand=True, fill=BOTH)

        frame = Frame(master=master, borderwidth=1, padding=5, relief=SUNKEN)
        frame.pack(expand=True, fill=BOTH)

        # the y scroll bar should extend the entire dialog so pack it first
        y_scrollbar = Scrollbar(frame)
        y_scrollbar.pack(side=RIGHT, fill=Y)

        default_font = nametofont("TkDefaultFont")
        make_font = make_font_builder(default_font)

        # use a centered label to say select something
        title_font = make_font(size=default_font.cget("size") + 2) if os.name == 'nt' else default_font
        title = Label(frame, text="Select a color.", font=title_font)
        title.pack(side=TOP, pady=(5, 10))

        # the canvas occupies the rest of the frame
        text_font = default_font if os.name == 'nt' else make_font(size=default_font.cget("size") - 2)
        canvas = Canvas(frame, bd=0, yscrollcommand=y_scrollbar.set)
        canvas.pack(expand=TRUE, fill=BOTH)

        canvas.config(yscrollcommand=y_scrollbar.set)
        y_scrollbar.config(command=canvas.yview)

        # the vertical spacing is based on the default font line space
        default_font_line_space = default_font.metrics("linespace")

        # column metrics
        overall_column_width = 100
        color_box_width = int(overall_column_width / 2)
        color_box_height = default_font_line_space * 2
        label_separation = int(default_font_line_space / 2)
        label_height = default_font_line_space
        column_height = color_box_height + label_separation + label_height

        # filter out duplicate colors
        color_filter: Set[str] = set()
        colors: List[Color] = []
        for color in get_colors():
            if color.to_hex() not in color_filter:
                colors.append(color)
                color_filter.add(color.to_hex())

        columns = 5
        row_separation = default_font_line_space

        # upper left hand corner of the color picker
        origin = Coord(10, 5)

        # setup the window size
        size_x = (columns * (overall_column_width + 12)) + origin.x
        size_y = (8 * (column_height + row_separation)) + origin.y
        self.minsize(size_x, size_y)
        self.maxsize(size_x, size_y)

        def color_selected(_iid: str, _color: Color):
            self._selected_color = _color

            # remove any previous select indicator
            _selected_tag = "color_selected"
            canvas.delete(_selected_tag)

            # now make the color look selected
            _lhs_x, _lhs_y, _rhs_x, _rhs_y = canvas.coords(_iid)
            canvas.create_line(_lhs_x - 4, _lhs_y - 4,
                               _lhs_x - 4, _rhs_y + 5,
                               _rhs_x + 5, _rhs_y + 5,
                               _rhs_x + 5, _lhs_y - 4,
                               _lhs_x - 4, _lhs_y - 4,
                               fill="dim gray", width=2, tags=_selected_tag)

        def add_color(_origin: Coord, _color: Color) -> str:

            # create the color box
            _hex_color = _color.to_hex()
            _lhs_x = _origin.x - int(color_box_width / 2)
            _rhs_y = _origin.y + color_box_height
            _box_iid = canvas.create_rectangle(_lhs_x, _origin.y, _lhs_x + color_box_width, _rhs_y,
                                               fill=_hex_color, outline=_hex_color,
                                               activeoutline="black", activewidth=1.5,
                                               state=NORMAL)

            # create the color label
            _text_iid = canvas.create_text(_origin.x, _origin.y + color_box_height + label_separation,
                                           font=text_font, text=color.name,
                                           anchor=N, justify=CENTER)

            # bind clicking on the text or the box to the color select callback
            canvas.tag_bind(_box_iid, "<1>", lambda e: color_selected(_box_iid, _color))
            canvas.tag_bind(_text_iid, "<1>", lambda e: color_selected(_box_iid, _color))

            return _box_iid

        column_center = int(overall_column_width / 2)
        selected_origin = origin
        selected_color_hex = self._selected_color.to_hex() if self._selected_color else ""
        for idx, color in enumerate(colors):

            # if this isn't the first pass, set the origin to the next row
            offset = idx % columns
            if idx and not offset:
                origin = origin.with_y_offset(column_height + row_separation)

            box_iid = add_color(origin.with_x_offset((offset * overall_column_width) + column_center), color)
            if color.to_hex() == selected_color_hex:
                color_selected(box_iid, color)
                selected_origin = origin

        # setup the scroll region
        bbox = canvas.bbox(ALL)
        canvas.config(scrollregion=(0, 0, bbox[2], bbox[3]))

        # position the view to the selected color if it isn't visible of the screen
        y_view = selected_origin.y - (4 * column_height)
        if y_view > 0:
            canvas.yview_moveto(y_view / bbox[3])

        canvas.bind_all("<MouseWheel>", lambda e: canvas.yview_scroll(-1 * (e.delta // 120), "units"))


class SettingsDialog(WeatherDialog):

    def __init__(self, master):
        self.domain_settings: Optional[SettingsDialog.DomainSettings] = None
        self.gui_settings: Optional[SettingsDialog.GuiSettings] = None
        self.log_settings: Optional[SettingsDialog.LogSettings] = None

        self.post_actions: List[Tuple[Callable[[], bool], Callable[[], None]]] = []

        self._gui_config: Dict[str, SettingValue] = get_settings("gui")
        self._domain_config: Dict[str, SettingValue] = get_settings("domain")
        super().__init__(master, title="Weather Data Settings")

    @staticmethod
    def mk_frame(master, label: str) -> LabelFrame:
        frame = tk_LabelFrame(master, text=label, relief=GROOVE)
        frame.grid(sticky=NSEW, padx=5, pady=5)
        frame.rowconfigure(0, weight=1)

        # try to force right side alignment of column for all the frames
        frame.columnconfigure(0, weight=1, minsize=130)

        # try to force the right column consume most of the row
        frame.columnconfigure(1, weight=10)

        # PyCharm is confused between LabelFrame and Labelframe (same thing with ttk different spelling)
        # noinspection PyTypeChecker
        return frame

    @staticmethod
    def mk_logging_settings(master, info: IntVar, debug: IntVar,
                            text: str = "Logging Level:", log_level: str = None, row=1):
        def set_checked_state():
            if info.get():
                debug_cb.configure(state=NORMAL)
            else:
                debug.set(0)
                debug_cb.configure(state=DISABLED)

        if log_level:
            if log_level == "info":
                info.set(1)
            elif log_level == "debug":
                info.set(1)
                debug.set(1)

        Label(master, text=text).grid(row=row, column=0, sticky=E, padx=(5, 1), pady=(5, 0))
        info_cb = Checkbutton(master, text="Include informational messages.", variable=info, command=set_checked_state)
        info_cb.grid(row=row, column=1, sticky=W, pady=(6, 0))

        debug_cb = Checkbutton(master, text="Include debug messages.", variable=debug, command=set_checked_state)
        debug_cb.grid(row=row + 1, column=1, sticky=W)

        set_checked_state()

    @staticmethod
    def mk_label(master, text: str, row: int, column: int = 0, sticky: str = E, pad_x=(5, 1), pad_y=(3, 2)) -> Label:
        label = Label(master, text=text, justify=RIGHT)
        label.grid(row=row, column=column, sticky=sticky, padx=pad_x, pady=pad_y)
        return label

    @staticmethod
    def set_setting_if_changed(section: SettingName, key: SettingName, value: SettingValue, case_fold: bool = False):
        if isinstance(value, bool):
            old_value = get_bool_setting(section, key)
        else:
            old_value = get_setting(section, key)
            if case_fold and isinstance(old_value, str):
                old_value = old_value.casefold()
        if value != old_value:
            set_setting(section, key, value)

    def body(self, master):

        # setup the color picker window
        master.pack(expand=True, fill=BOTH)

        # create the canvas all the settings will be put onto
        settings_container = Canvas(master, highlightthickness=0)

        canvas_frames: List[str] = []

        origin = Coord(0, 0)
        log_frame = self.mk_frame(master, " Logging ")
        self.log_settings = SettingsDialog.LogSettings(log_frame)
        self.post_actions.append((self.log_settings.validate, self.log_settings.apply))
        canvas_frames.append(settings_container.create_window(origin, window=log_frame, anchor=NW))

        origin = origin.with_y_offset(log_frame.winfo_height() + 10)
        domain_frame = self.mk_frame(master, " Domain ")
        self.domain_settings = SettingsDialog.DomainSettings(domain_frame)
        self.post_actions.append((self.domain_settings.validate, self.domain_settings.apply))
        canvas_frames.append(settings_container.create_window(origin, window=domain_frame, anchor=NW))

        origin = origin.with_y_offset(domain_frame.winfo_height() + 10)
        gui_frame = self.mk_frame(master, " UI ")
        self.gui_settings = SettingsDialog.GuiSettings(gui_frame)
        self.post_actions.append((self.gui_settings.validate, self.gui_settings.apply))
        canvas_frames.append(settings_container.create_window(origin, window=gui_frame, anchor=NW))

        y_scrollbar = Scrollbar(master, orient=VERTICAL)

        # use pack, there's only 2 widgets
        y_scrollbar.pack(side=RIGHT, fill=Y)
        settings_container.pack(fill=BOTH, expand=True, padx=5)

        # bind the scroll bar
        y_scrollbar.configure(command=settings_container.yview)
        settings_container.configure(yscrollcommand=y_scrollbar.set)

        def settings_frame_width(_event):
            _canvas_width = _event.width
            for item in canvas_frames:
                settings_container.itemconfigure(item, width=_canvas_width)
            log.debug("canvas width={} dialog size={}".format(_canvas_width, (self.winfo_width(), self.winfo_height())))

        settings_container.bind("<Configure>", settings_frame_width)

        # a silly way to set the scrolling region of the settings canvas
        # def canvas_scroll_region(event):
        #     bbox = settings.bbox(ALL)
        #     settings.configure(scrollregion=settings.bbox(ALL))
        #     print("scrollregion={}".format(bbox))
        # domain_settings.bind("<Configure>", canvas_scroll_region)

        settings_container.configure(scrollregion=settings_container.bbox(ALL))

        master.update()
        dialog_width = max(domain_frame.winfo_width(), gui_frame.winfo_width()) + 40
        dialog_height = domain_frame.winfo_height() + gui_frame.winfo_height() + 60
        settings_geometry = "{}x{}".format(dialog_width, dialog_height)
        self.geometry(settings_geometry)
        log.debug("settings geometry={}".format(settings_geometry))
        self.minsize(width=dialog_width, height=dialog_height)

    def buttonbox(self):
        box = Frame(self)
        Button(box, text="Save", width=10, command=self.ok).pack(side=LEFT, padx=5, pady=5)
        Button(box, text="Cancel", width=10, command=self.cancel, default=ACTIVE).pack(side=LEFT, padx=5, pady=5)
        # self.bind("<Return>", self.ok)
        self.bind("<Escape>", self.cancel)
        box.pack()
        box.update()
        # todo: look into setting the dialog size here instead of in the body

    @staticmethod
    def get_logging_level(info: int, debug: int) -> str:
        return "debug" if debug else "info" if info else "warning"

    def apply(self):
        super().apply()
        for _, applier in self.post_actions:
            applier()
        save_settings()

    def validate(self) -> bool:
        for validator, _ in self.post_actions:
            if not validator():
                return False
        return True

    class DomainSettings:

        keys = Enum('keys', [(k.casefold(), k.casefold()) for k in ["domain",
                                                                    "weather_data_dir",
                                                                    "history_api_key",
                                                                    "logging_level"]])

        def __init__(self, master: LabelFrame):
            self.set_if_changed = SettingsDialog.set_setting_if_changed
            self.get_logging_level = SettingsDialog.get_logging_level

            row = 0
            SettingsDialog.mk_label(master, "Default weather data\ndirectory name:", row)

            # the weather data directory uses 2 widgets so drop them in a frame
            dir_widgets = Frame(master)
            dir_widgets.grid(row=row, column=1, sticky=NSEW, pady=(2, 0))

            dir_name_validator = master.register(self.validate_dir_name)
            self.weather_data_dir = StringVar()
            self.weather_data_dir.set(get_setting(self.keys.domain, self.keys.weather_data_dir))
            dir_entry = Entry(dir_widgets,
                              width=30,
                              textvariable=self.weather_data_dir,
                              validatecommand=(dir_name_validator, '%P'),
                              validate='focusout')
            dir_entry.pack(side=LEFT, padx=(1, 5))

            def reset_dir_name():
                _default_dir = get_setting(self.keys.domain, self.keys.weather_data_dir)
                if _default_dir:
                    self.weather_data_dir.set(_default_dir)

            Button(dir_widgets, text="Reset to default", command=reset_dir_name).pack(side=LEFT, padx=5)

            row += 1
            SettingsDialog.mk_label(master, "History API Provider key:", row)
            self.history_api_key = StringVar()
            self.history_api_key.set(get_setting(self.keys.domain, self.keys.history_api_key))
            entry = Entry(master, width=35, textvariable=self.history_api_key)
            entry.grid(row=row, column=1, padx=(0, 5), sticky=W)

            row += 1
            self.info = IntVar(0)
            self.debug = IntVar(0)
            logging_level = get_setting(self.keys.domain, self.keys.logging_level)
            SettingsDialog.mk_logging_settings(master,
                                               info=self.info,
                                               debug=self.debug,
                                               log_level=logging_level,
                                               row=row)

            master.update()

        def validate_dir_name(self, dir_name: str) -> bool:
            valid = is_pathname_valid(dir_name)
            if not valid:
                messagebox.showerror("Domain Settings", message="The Weather Data pathname is not valid!")
                self.weather_data_dir.set(get_setting(self.keys.domain, self.keys.weather_data_dir))
            return valid

        def validate(self):
            return self.validate_dir_name(self.weather_data_dir.get())

        def apply(self):
            self.set_if_changed(self.keys.domain, self.keys.weather_data_dir, self.weather_data_dir.get())
            self.set_if_changed(self.keys.domain, self.keys.history_api_key, self.history_api_key.get())
            logging_level = self.get_logging_level(self.info.get(), self.debug.get())
            self.set_if_changed(self.keys.domain, self.keys.logging_level, logging_level, case_fold=True)

    class GuiSettings:

        keys = Enum("keys", [(k.casefold(), k.casefold()) for k in ["gui",
                                                                    "logging_level",
                                                                    "windows_native_theme",
                                                                    "graph_colors"]])

        def __init__(self, master: LabelFrame):
            self.set_if_changed = SettingsDialog.set_setting_if_changed
            self.get_logging_level = SettingsDialog.get_logging_level

            # use windows native theme?
            row = 0
            use_native_theme = 1 if get_setting(self.keys.gui, self.keys.windows_native_theme) else 0
            self.windows_native_theme = IntVar(value=use_native_theme)
            if os.name == 'nt':
                SettingsDialog.mk_label(master, text="Use Windows Native\ntheme:", row=row)
                Checkbutton(master,
                            text="(takes effect on restart)",
                            variable=self.windows_native_theme).grid(row=row, column=1, sticky=S + W)
                row += 1

            # select colors
            label_text = "Daily Weather History\ngraph colors:\n(click color to change)"""
            label = SettingsDialog.mk_label(master, text=label_text, row=row)
            label.grid(rowspan=5)

            colors = get_colors()
            self._color_by_name: Dict[str, Color] = {c.name.casefold(): c for c in colors}
            self._color_by_hex: Dict[str, Color] = {c.to_hex(): c for c in colors}

            # there are 5 buttons and text labels to represent colors so put them in a frame
            color_name_slots = 5
            colors_frame = Frame(master)
            colors_frame.grid(row=row, column=1, rowspan=color_name_slots, sticky=NSEW, pady=(2, 0))
            colors_frame.columnconfigure(0, weight=1)
            colors_frame.columnconfigure(1, weight=100)

            color_names: List[str] = get_setting(self.keys.gui, self.keys.graph_colors)
            len_color_names = len(color_names)
            if color_name_slots > len_color_names:
                log.warning("%d colors are excepted, only %d were listed.", color_name_slots, len_color_names)
                color_names += ["black" * (color_name_slots - len_color_names)]
            elif color_name_slots < len_color_names:
                log.warning("Only %d colors are excepted, not %d.", color_name_slots, len_color_names)
                color_names = color_names[:color_name_slots]

            for idx, color in enumerate(color_names):
                if color.casefold() not in self._color_by_name and color not in self._color_by_hex:
                    log.warning("'%s' is not an accepted color. See X11 colors text for the list of colors.", color)
                    color_names[idx] = "black"

            self._colors: List[Tuple[StringVar, tk_Button]] = []
            for idx, color_name in enumerate(color_names):
                button = tk_Button(colors_frame, width=5)
                button.grid(row=idx, column=0, sticky=W, padx=(2, 5), pady=(3, 0))

                color = self.get_color(color_name)
                self.set_button_color(button, color)

                color_var = StringVar(value=color.name)
                Label(colors_frame, textvariable=color_var).grid(row=idx, column=1, sticky=W)

                self._colors.append((color_var, button))
                button.configure(command=self.get_color_picker(master, color_var, button))
            row += color_name_slots

            # allow the colors to be reset to default values
            reset_frame = Frame(master)
            reset_frame.grid(row=row, column=1, padx=(2, 0), pady=(3, 0), sticky=NSEW)
            Button(reset_frame, command=self.reset_colors, text="Reset colors to default.").grid(sticky=NSEW)
            row += 1

            # logging levels
            self.info = IntVar(0)
            self.debug = IntVar(0)
            SettingsDialog.mk_logging_settings(master,
                                               info=self.info,
                                               debug=self.debug,
                                               log_level=get_setting(self.keys.gui, self.keys.logging_level),
                                               row=row)
            master.update()

        @staticmethod
        def set_button_color(button: tk_Button, color: Color):
            hex_color = color.to_hex()
            button.configure(background=hex_color, activebackground=hex_color)

        @staticmethod
        def validate():
            return True

        def apply(self):
            windows_native_theme = True if self.windows_native_theme.get() else False
            self.set_if_changed(self.keys.gui, self.keys.windows_native_theme, windows_native_theme)
            button_colors = [bc.get() for bc, _ in self._colors]
            self.set_if_changed(self.keys.gui, self.keys.graph_colors, button_colors)
            logging_level = self.get_logging_level(self.info.get(), self.debug.get())
            self.set_if_changed(self.keys.gui, self.keys.logging_level, logging_level, case_fold=True)

        def get_color_picker(self, master, color_name: StringVar, button: tk_Button) -> Callable[[None], None]:
            def color_picker():
                color = self.get_color(color_name.get())
                dialog = ColorPicker(master, color)
                if not dialog.canceled:
                    color = dialog.selected_color
                    color_name.set(color.name)
                    self.set_button_color(button, color)

            return color_picker

        def get_color(self, color_name: str) -> Color:
            color = self._color_by_name.get(color_name.casefold())
            if not color:
                color = self._color_by_hex.get(color_name.casefold())
            return color if color else self._color_by_name.get("black")

        def reset_colors(self):
            default_colors = get_default_setting(self.keys.gui, self.keys.graph_colors)
            if default_colors:
                for idx, color_name in enumerate(default_colors):
                    _color = self.get_color(color_name)
                    color_name, _button = self._colors[idx]
                    color_name.set(_color.name)
                    self.set_button_color(_button, _color)

    class LogSettings:

        keys = Enum("keys", [(k.casefold(), k.casefold()) for k in ["logging",
                                                                    "log_format",
                                                                    "default_logging_level"]])

        def __init__(self, master: LabelFrame):
            self.set_if_changed = SettingsDialog.set_setting_if_changed
            self.get_logging_level = SettingsDialog.get_logging_level

            row = 0

            SettingsDialog.mk_label(master, "log format\n(restart required):", row)

            # the log format uses 2 widgets so drop them in a frame
            format_widgets = Frame(master)
            format_widgets.grid(row=row, column=1, sticky=NSEW, pady=(2, 0))

            self.log_format = StringVar()
            self.log_format.set(get_setting(self.keys.logging, self.keys.log_format))
            format_entry = Entry(format_widgets,
                                 width=35,
                                 textvariable=self.log_format)
            format_entry.pack(side=LEFT, padx=(1, 5))

            def reset_log_format():
                default_log_format = get_setting(self.keys.logging, self.keys.log_format)
                self.log_format.set(default_log_format)

            Button(format_widgets, text="Reset", command=reset_log_format).pack(side=LEFT, padx=5)

            row += 1
            self.info = IntVar(0)
            self.debug = IntVar(0)
            logging_level = get_setting(self.keys.logging, self.keys.default_logging_level)
            SettingsDialog.mk_logging_settings(master,
                                               text="Default Logging Level:",
                                               info=self.info,
                                               debug=self.debug,
                                               log_level=logging_level,
                                               row=row)

            master.update()

        @staticmethod
        def validate():
            return True

        def apply(self):
            self.set_if_changed(self.keys.logging, self.keys.log_format, self.log_format.get())
            logging_level = self.get_logging_level(self.info.get(), self.debug.get())
            self.set_if_changed(self.keys.logging, self.keys.default_logging_level, logging_level, case_fold=True)
