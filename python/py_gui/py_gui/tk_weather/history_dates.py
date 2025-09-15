import tkinter as tk
import tkinter.ttk as ttk
from collections import namedtuple
from copy import copy
from datetime import date
from tkinter import messagebox
from tkinter.font import nametofont
from typing import Callable, List, NamedTuple, Optional

from py_weather_lib import PyDateRange, PyHistoryDates, PyLocationFilters

from .add_history import AddHistory
from .history_graph import HistoryGraph
from .history_report import HistoryReport
from .infrastructure import Stopwatch, WeatherEvent, WeatherView
from ..config import get_logger
from ..domain import WeatherData

__all__ = ['HistoriesDates']
log = get_logger(__name__)


def _warn(message: str):
    """
    Used to display a warning message.
    """
    messagebox.showwarning('History Dates', message)


def _error(message: str, error: SystemError):
    """
    Used to log and display error messages.
    """
    log.error('%s\n%s', message, error)
    messagebox.showerror('History Dates', message)


class HistoryDatesSelection(NamedTuple):
    """
    Holds the location history dates indexes selected in the HistoryDatesView.
    """
    locations_idx: List[int] = []
    date_ranges_idx: List[int] = []


class HistoriesDates(WeatherView):
    def __init__(self, parent, weather_data: WeatherData, add_tab: Callable[[str, WeatherView], None],
                 location_limit=5, date_ranges_limit=5):
        super().__init__()
        self._weather_data = weather_data
        self._view = HistoryDatesView(parent)
        self._view.bind_event('<Button-3>', self._right_click)
        self._parent = parent
        self._add_tab = add_tab

        # make sure you have location history dates
        self._locations_history_dates: List[PyHistoryDates] = []
        self.refresh()

        self._locations_limit = max(1, location_limit)
        self._date_ranges_limit = max(1, date_ranges_limit)
        self._previous_selection = HistoryDatesSelection()

    def view(self) -> tk.Frame:
        return self._view

    def refresh(self):
        try:
            elapsed = Stopwatch()
            self._locations_history_dates = self._weather_data.backend.get_history_dates(PyLocationFilters([]))
            log.info('refresh location history dates %s', elapsed)
            self._view.refresh(self._locations_history_dates)
        except SystemError as error:
            _error('An error occurred getting history dates.', error)

    # noinspection DuplicatedCode
    def _validate_selection(self, selection: HistoryDatesSelection) -> bool:
        """
        Check that the selection made in the HistoryDatesView appears to be okay.
        """
        try:
            locations_selected = len(selection.locations_idx)
            if locations_selected > self._locations_limit:
                previously_selected = len(self._previous_selection.locations_idx)
                if locations_selected != previously_selected:
                    # check to see if more locations have been selected
                    if previously_selected <= self._locations_limit:
                        _warn(f'Only {self._locations_limit} locations can be selected at the same time.')
                    elif locations_selected > previously_selected:
                        _warn(f'Tool many locations are selected! The limit is {self._locations_limit}.')
                    return False

            date_ranges_selected = len(selection.date_ranges_idx)
            if date_ranges_selected > self._date_ranges_limit:
                previously_selected = len(self._previous_selection.date_ranges_idx)
                if date_ranges_selected != previously_selected:
                    # check to see if more date ranges have been selected
                    if previously_selected <= self._date_ranges_limit:
                        _warn(f'Only {self._date_ranges_limit} history dates can be selected at the same time.')
                    elif date_ranges_selected > previously_selected:
                        _warn(f'Tool many dates ranges are selected! The limit is {self._locations_limit}.')
                    return False

            # until year over year is implemented there can only be 1 date range
            if date_ranges_selected > 1:
                _warn(f'Currently only 1 of the history dates can be selected.')
                return False

        finally:
            self._previous_selection = selection
            return True

    def _right_click(self, event: tk.Event):
        """Validate the selection and dispatch the context menu."""

        selection = self._view.get_selection()
        if not selection or not self._validate_selection(selection):
            return

        # create the popup menu
        popup_menu = tk.Menu(self._parent, tearoff=0)
        if len(selection.locations_idx) == 1:
            if not len(selection.date_ranges_idx):
                popup_menu.add_command(label='Add History', command=self._add_history)
            popup_menu.add_command(label="History Reports", command=self._history_report)
        popup_menu.add_command(label="History Graphs", command=self._history_graph)
        try:
            popup_menu.tk_popup(self._view.winfo_rootx() + event.x, self._view.winfo_rooty() + event.y)
        finally:
            popup_menu.grab_release()

    def _add_history(self):
        print('HistoryDates _add_history')
        # the popup menu will only call this when 1 location is selected
        selection = self._view.get_selection()
        index = int(selection.locations_idx[0])
        location = self._locations_history_dates[index].location
        add_history = AddHistory(self._view, location, self._weather_data)
        if add_history.is_history_added:
            self._parent.event_generate(WeatherEvent.REFRESH_VIEW)

    def _history_report(self):
        """
        called from the context menu to launch the HistoryReport. It assumes the selection
        has been validated.
        """
        selection = self._view.get_selection()
        if not selection:
            return

        if len(selection.locations_idx) > 1:
            _warn('History reports can only use 1 location.')
            return
        if len(selection.date_ranges_idx) > 1:
            _warn('History reports can only use 1 history date range.')
            return

        # make sure there is an initial date range
        location_history_dates = self._locations_history_dates[selection.locations_idx[0]]
        if not selection.date_ranges_idx:
            date_range = location_history_dates.history_dates[-1]
        else:
            # date_range = selection.date_ranges_idx[0]
            date_range = location_history_dates.history_dates[selection.date_ranges_idx[0]]

        history_report = HistoryReport(self._parent, location_history_dates.location.alias, self._weather_data,
                                       date_range)
        if history_report:
            self._add_tab(f'{location_history_dates.location.name}', history_report)

    def _history_graph(self):
        """
        Called from the context menu to launch the HistoryGraph. It assumes the selection
        has been validated.
        """
        selection = self._view.get_selection()
        if not selection:
            return

        if len(selection.date_ranges_idx) > 1:
            _warn('History graphs can only use 1 history date range.')
            return

        locations_history_dates = [self._locations_history_dates[index] for index in selection.locations_idx]
        locations = [lhd.location for lhd in locations_history_dates]
        if not selection.date_ranges_idx:
            date_range = None
        else:
            date_range = locations_history_dates[0].history_dates[selection.date_ranges_idx[0]]
        HistoryGraph(self._parent, locations, self._weather_data, self._add_tab, date_range)


class HistoryDatesView(tk.Frame):
    """A Treeview column definition."""
    __Column = namedtuple('Column', ['iid', 'text', 'heading_anchor', 'column_anchor', 'stretch'])

    def __init__(self, parent, **kwargs):
        super().__init__(parent, **kwargs)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)

        scrollbar = tk.Scrollbar(self)
        scrollbar.grid(row=0, column=1, sticky=tk.N + tk.S + tk.E)

        # the view columns
        self._columns = [
            HistoryDatesView.__Column("#0", "Location", tk.CENTER, tk.W, tk.NO),
            HistoryDatesView.__Column("history_dates", "History Dates", tk.CENTER, tk.CENTER, tk.NO),
        ]

        ids = [cid.iid for cid in self._columns[1:]]
        self._tree = ttk.Treeview(self, columns=ids, yscrollcommand=scrollbar.set)
        self._tree.grid(row=0, column=0, sticky=tk.NSEW)

        scrollbar.config(command=self._tree.yview)

        heading_font = nametofont(ttk.Style().lookup(f'{self._tree.winfo_class()}.Heading', 'font'))
        pad = heading_font.measure('0' * 2)
        self._heading_widths = [pad + heading_font.measure(column.text) for column in self._columns]

        for column in self._columns:
            self._tree.heading(column.iid, text=column.text, anchor=column.heading_anchor)
            self._tree.column(column.iid, stretch=column.stretch, anchor=column.column_anchor)

        self.bind_event('<<TreeviewSelect>>', self._on_selection)
        self.bind_event('<ButtonRelease-1>', self._select_parent_for_date_range_at)
        self.bind_event('<Button-3>', self._right_click)

    def bind_event(self, event: str, action: Callable[[tk.Event], None]):
        """
        Binds an event to the internally managed Treeview.
        """
        self._tree.bind(event, action, add='+')

    def refresh(self, history_dates: List[PyHistoryDates]):
        """
        Replace existing Treeview content with the locations history dates.
        """
        # clear the tree
        for item in self._tree.get_children():
            self._tree.delete(item)

        # grab the font measure tool
        measure = nametofont(ttk.Style().lookup(self._tree.winfo_class(), 'font')).measure

        # create the tree contents
        history_date_rows = [HistoryDatesRow(lhd) for lhd in history_dates]

        # get the column widths
        pad = measure('0' * 4)
        column_widths = copy(self._heading_widths)
        for history_dates_row in history_date_rows:
            column_widths[0] = max(column_widths[0], measure(history_dates_row.name) + pad)
            if history_dates_row.history_dates:
                max_date_range_width = max([measure(str(dr)) for dr in history_dates_row.history_dates])
            else:
                max_date_range_width = 0
            column_widths[1] = max(column_widths[1], max_date_range_width + pad)

        # set the column widths
        self._tree.column(self._columns[0].iid, minwidth=column_widths[0], width=column_widths[0])
        self._tree.column(self._columns[1].iid, minwidth=column_widths[1], width=column_widths[1])

        # create the tree contents
        for index, history_date_row in enumerate(history_date_rows):
            # the order of the locations history dates determines the iid of each location
            pid = str(index)
            self._tree.insert('', 'end', iid=pid, text=history_date_row.name)
            for history_date in history_date_row.history_dates:
                self._tree.insert(pid, 'end', values=[history_date])

        # set focus to the first item in the tree
        self._tree.focus_set()
        self._tree.focus(0)

    def get_selection(self) -> Optional[HistoryDatesSelection]:
        """
        Get the locations history dates indexes from the Treeview selection.
        """
        selection = self._tree.selection()
        if not selection:
            return None

        # get the selected location indexes
        locations_idx = [int(iid) for iid in selection if not self._tree.parent(iid)]

        # get the selected date ranges
        location_date_ranges: (int, int) = []
        for iid in selection:
            pid = self._tree.parent(iid)
            if pid:
                location_date_ranges.append((int(pid), self._tree.index(iid)))

        if location_date_ranges:
            # date ranges can only come from 1 location
            (pid, _) = location_date_ranges[0]
            for (iid, _) in location_date_ranges[1:]:
                if iid != pid:
                    _warn('The date range selection can only come from 1 location.')
                    return None
            # make sure the date ranges location is first
            if pid != locations_idx[0]:
                locations_idx.pop(locations_idx.index(pid))
                locations_idx.insert(0, pid)

        return HistoryDatesSelection(locations_idx, [idx for (_, idx) in location_date_ranges])

    def _item_at(self, x: int, y: int) -> str:
        return self._tree.identify('item', x, y)

    def _on_open(self, event: tk.Event):
        pass

    def _on_open_close(self, _):
        """
        Turn off the location being selected in the Treeview when it is opened or closed.
        """

        # the selection will contain the location that was opened or closed
        selection = self._tree.selection()
        if selection:
            self._tree.selection_remove(selection)

    def _on_selection(self, _):
        """
        Called when there is a selection made in the Treeview.
        """
        selection = self._tree.selection()
        if not selection:
            return

        # remove the child date ranges whose parent is not selected
        for iid in selection:
            pid = self._tree.parent(iid)
            if pid and not self._tree.item(pid, 'open'):
                self._tree.selection_remove(iid)

        # now go back through and make sure the date ranges parent is selected
        selection = self._tree.selection()
        for iid in selection:
            pid = self._tree.parent(iid)
            if pid:
                try:
                    selection.index(pid)
                except ValueError:
                    self._tree.selection_set(selection + (pid,))

                # there can be only 1 location for the child date ranges so you are done
                break

    def _select_parent_for_date_range_at(self, event: tk.Event):
        """
        Called when the left mouse button is released to make sure the
        parent of a selected date range is also selected.
        """
        # get the item associated with the coordinates
        item = self._item_at(event.x, event.y)
        if item:
            # check to see if item at the x,y is a child of a location
            pid = self._tree.parent(item)
            if pid:
                selection = self._tree.selection()
                try:
                    selection.index(pid)
                except ValueError:
                    # if the parent is not part of the selection add it
                    self._tree.selection_set(selection + (pid,))

    def _right_click(self, event: tk.Event):
        item = self._item_at(event.x, event.y)
        if item:
            selection = self._tree.selection()
            if not selection:
                self._tree.selection_set([item])
            elif item not in selection:
                self._tree.selection_set(selection + (item,))
                self._select_parent_for_date_range_at(event)


class HistoryDatesRow:
    """
    Holds the Treeview rows associated with a LocationHistoryDates.
    """

    def __init__(self, history_dates: PyHistoryDates):
        self.iid = history_dates.location.alias
        self.name = history_dates.location.name
        self.history_dates = [HistoryDatesRow.__describe_date_range(dr) for dr in history_dates.history_dates]

    @staticmethod
    def __describe_date(d: date) -> str:
        return d.strftime('%b %d, %Y')

    @staticmethod
    def __describe_date_range(date_range: PyDateRange) -> str:
        start = HistoryDatesRow.__describe_date(date_range.start)
        if date_range.start == date_range.end:
            return start
        return f'{start} thru {HistoryDatesRow.__describe_date(date_range.end)}'
