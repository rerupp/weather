import tkinter as tk
from copy import copy
from tkinter import messagebox
from tkinter.simpledialog import Dialog
from typing import List, Optional

from py_weather_lib import PyCityFilter, PyLocation

from .add_location import AddLocation
from .infrastructure import (Stopwatch, WeatherEvent, WeatherView)
from .widgets import LocationsView
from ..config import get_logger
from ..domain import WeatherData

__all__ = ['CitySearch']
log = get_logger(__name__)


class CitySearch(WeatherView):
    """The search locations results view"""

    def __init__(self, parent, weather_data: WeatherData):
        super().__init__()
        self._parent = parent
        self._weather_data = weather_data

        # create the locations view
        self._view = LocationsView(parent, hide_alias=True, multi_select=False)
        self._view.bind_event('<F3>', lambda _: self._get_search_criteria())
        self._view.bind_event('<Button-3>', self._popup_menu)

        def add_location(event: tk.Event):
            selection = self._view.selection_at(event.x, event.y)
            if selection:
                self._view.set_selection([selection])
            self._add_location()

        self._view.bind_event('<Return>', add_location)

        # initialize the search metrics
        self._location_criteria = PyCityFilter(limit=25)
        self._get_search_criteria()

        self._locations: List[PyLocation] = []
        self._refresh()

    def view(self) -> tk.Frame:
        return self._view

    def _refresh(self):
        stopwatch = Stopwatch()
        try:
            self._locations = self._weather_data.backend.search_locations(self._location_criteria)
            self._view.refresh(self._locations)
        except SystemError as error:
            msg = 'There was an error searching for US cities.'
            log.error(f'{msg}:\n{error}')
            messagebox.showerror('Search Cities', f'{msg}\nCheck the log for more information.')
        log.debug('%s refresh %s', self.__class__.__name__, stopwatch)

    def _get_search_criteria(self):
        if not GetSearchCriteria(self._view, self._location_criteria).is_canceled():
            self._refresh()

    def _add_location(self):
        selection = self._view.get_selection()
        if not selection:
            log.warning(f'No location selected for {self._location_criteria}')
            return
        location = self._locations[selection[0]]
        if location:
            if not AddLocation(self._view, copy(location), self._weather_data).is_cancelled():
                self._parent.event_generate(WeatherEvent.REFRESH_VIEW)

    def _popup_menu(self, event: tk.Event):

        mouse_iid = self._view.selection_at(event.x, event.y)
        if mouse_iid:
            self._view.set_selection([mouse_iid])

        (x, y) = self._view.context_xy(event)
        popup_menu = tk.Menu(self._parent, tearoff=0)
        popup_menu.add_command(label="Add Location", command=self._add_location)
        popup_menu.add_command(label="New Search", command=self._get_search_criteria)
        try:
            popup_menu.tk_popup(x, y)
        finally:
            popup_menu.grab_release()


class GetSearchCriteria(Dialog):
    """The dialog that lets the search criteria be set."""

    def __init__(self, parent, city_filter: PyCityFilter):
        self._is_canceled = True
        self._city_filter = city_filter

        # set up the field attributes
        self._name: Optional[tk.Entry] = None
        self._name_value: Optional[tk.StringVar] = None

        self._state: Optional[tk.Entry] = None
        self._state_value: Optional[tk.StringVar] = None

        self._zip_code: Optional[tk.Entry] = None
        self._zip_code_value: Optional[tk.StringVar] = None

        self._limit: Optional[tk.Entry] = None
        self._limit_value: Optional[tk.StringVar] = None

        # kick off the dialog
        super().__init__(parent, title='Search Criteria')

    def body(self, parent: tk.Frame) -> tk.Entry:
        """Add the search criteria fields to the Dialog."""

        def mk_entry(row: int, label: str, variable: tk.StringVar, entry_len: int) -> tk.Entry:
            tk.Label(master=parent, text=label).grid(row=row, column=0, sticky=tk.E, padx=(5, 2), pady=5)
            entry = tk.Entry(parent, width=entry_len, textvariable=variable)
            entry.grid(row=row, column=1, sticky=tk.W, padx=(0, 5), pady=5)
            return entry

        value_or_empty = lambda v: v if v else ''

        self._name_value = tk.StringVar(parent, value=value_or_empty(self._city_filter.name))
        self._name = mk_entry(0, "Name:", self._name_value, 40)

        self._state_value = tk.StringVar(parent, value=value_or_empty(self._city_filter.state))
        self._state = mk_entry(1, "State:", self._state_value, 25)

        self._zip_code_value = tk.StringVar(parent, value=value_or_empty(self._city_filter.zip_code))
        self._zip_code = mk_entry(2, "Zip Code:", self._zip_code_value, 6)

        self._limit_value = tk.StringVar(parent, value=str(self._city_filter.limit))
        self._limit = mk_entry(3, "Limit:", self._limit_value, 5)

        def number_only(action, text):
            # 1 is the action code for insert
            if '1' == action:
                for c in text:
                    if not c.isdigit():
                        return False
            return True

        number_validator = self.register(number_only)
        # %d is the action code, %S is the text string
        self._zip_code.configure(validate="key", validatecommand=(number_validator, '%d', '%S'))
        self._limit.configure(validate="key", validatecommand=(number_validator, '%d', '%S'), justify=tk.RIGHT)

        return self._name

    def apply(self, event=None):
        """Move the contents of the dialog fields into the location criteria."""

        value_or_none = lambda v: v if v else None
        self._city_filter.name = value_or_none(self._name_value.get().strip())
        self._city_filter.state = value_or_none(self._state_value.get().strip())
        self._city_filter.zip_code = value_or_none(self._zip_code_value.get().strip())
        self._city_filter.limit = int(self._limit.get())
        self._is_canceled = False

    def is_canceled(self) -> bool:
        return self._is_canceled
