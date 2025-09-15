import tkinter as tk
import tkinter.messagebox as messagebox
from tkinter import *
from typing import (Callable, List)

from py_weather_lib import PyLocation, PyLocationFilters

from .add_history import AddHistory
from .history_graph import HistoryGraph
from .history_report import HistoryReport
from .infrastructure import Stopwatch, WeatherEvent, WeatherView
from .widgets import LocationsView
from ..config import get_logger
from ..domain import WeatherData

__all__ = ['Locations']
log = get_logger(__name__)


def _warn(msg: str):
    messagebox.showwarning(title='Locations View', message=msg)


def _error(msg: str, error: SystemError):
    log.error('%s.\n%s', msg, error)
    messagebox.showerror(title='Locations View', message=f'{msg}\nCheck the log for more information.')


class Locations(WeatherView):
    def __init__(self, parent, weather_data: WeatherData, add_tab: Callable[[str, WeatherView], None],
                 hide_alias=False, multi_select=True, select_limit=5):
        super().__init__()
        self._parent = parent
        self._weather_data = weather_data
        self._view = LocationsView(parent, hide_alias, multi_select)
        self._view.bind_event('<ButtonRelease-1>', self._left_click)
        self._view.bind_event('<Button-3>', self._right_click)
        self._add_tab = add_tab

        # if you cannot get the locations then indicate the view is cancelled
        self._locations: List[PyLocation] = []
        self.refresh()

        # remember the last selection so you can restore it if there's an issue
        self._previous_selection = self._view.get_selection()
        self._select_limit = select_limit

    def view(self) -> tk.Frame:
        return self._view

    def refresh(self):
        log.debug(f'{self.__class__.__name__} refresh')
        try:
            sw = Stopwatch()
            self._locations = self._weather_data.backend.get_locations(PyLocationFilters())
            self._view.refresh(self._locations)
            log.debug('refresh %s', sw)
        except SystemError as error:
            _error('There was an error loading locations.', error)

    def _left_click(self, _):
        selection = self._view.get_selection()
        selection_len = len(selection)
        if selection_len > self._select_limit:
            # check to see if the selection is being taken back
            if len(self._previous_selection) <= selection_len:
                messagebox.showwarning(title='Location Selection',
                                       message=f'Only {self._select_limit} locations can be selected.')
        self._previous_selection = selection

    def _right_click(self, event: Event):
        selection = self._view.get_selection()
        selection_len = len(selection)

        if selection_len > self._select_limit:
            messagebox.showwarning(title='Location Selection',
                                   message=f'Only {self._select_limit} locations can be selected.')
        elif selection_len == 1:
            selection_iid = self._view.selection_at(event.x, event.y)
            if selection_iid:
                self._view.set_selection([selection_iid])
            self._single_select_popup_menu(event)
        elif selection_len > 1:
            self._multi_select_popup_menu(event)

    def _single_select_popup_menu(self, event: Event):
        # make sure the mouse is over the view
        context_xy = self._view.context_xy(event)
        if not context_xy:
            return

        # all the popups need the selected location
        selection = self._view.get_selection()
        location = self._locations[selection[0]]

        popup_menu = Menu(self._parent, tearoff=0)

        # add history
        def add_history():
            if AddHistory(self._parent, location, self._weather_data).is_history_added:
                self._parent.event_generate(WeatherEvent.REFRESH_VIEW)

        popup_menu.add_command(label="Add History", command=add_history)

        # history reports
        def history_report():
            report = HistoryReport(self._parent, location.alias, self._weather_data)
            if report:
                self._add_tab(f'{location.name} Report', report)

        popup_menu.add_command(label="History Report", command=history_report)

        # history graph
        def history_graph():
            HistoryGraph(self._parent, location, self._weather_data, self._add_tab)

        popup_menu.add_command(label="History Graph", command=history_graph)

        # fire the popup menu
        (x, y) = context_xy
        try:
            popup_menu.tk_popup(x, y)
        finally:
            popup_menu.grab_release()

    def _multi_select_popup_menu(self, event: Event):
        # make sure the mouse is over the view
        context_xy = self._view.context_xy(event)
        if not context_xy:
            return

        # all the popups need the selected location
        selection = self._view.get_selection()
        locations = [self._locations[iid] for iid in selection]

        popup_menu = Menu(self._parent, tearoff=0)

        # history graph
        def history_graph():
            HistoryGraph(self._parent, locations, self._weather_data, self._add_tab)

        popup_menu.add_command(label="History Graph", command=history_graph)

        # fire the popup menu
        (x, y) = context_xy
        try:
            popup_menu.tk_popup(x, y)
        finally:
            popup_menu.grab_release()
