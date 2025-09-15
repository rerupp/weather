import tkinter as tk
import tkinter.ttk as ttk
from collections import namedtuple
from copy import copy
from tkinter import *
from tkinter.font import nametofont
from tkinter.ttk import *
from typing import Callable, List, Optional, Tuple

from py_weather_lib import PyLocation

from ...config import get_logger

__all__ = ['LocationsView']
log = get_logger(__name__)


class LocationsView(tk.Frame):
    # keep the tree view columns to yourself
    __Column = namedtuple('Column', ['iid', 'text', 'heading_anchor', 'column_anchor', 'stretch'])

    def __init__(self, parent, hide_alias: bool, multi_select: bool, **kwargs):
        # set up the frame the view will sit in
        tk.Frame.__init__(self, parent, **kwargs)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)

        # define the columns
        self._columns = [
            LocationsView.__Column("name", "Name", tk.CENTER, tk.W, tk.NO),
            LocationsView.__Column("alias", "Alias", tk.CENTER, tk.W, tk.NO),
            LocationsView.__Column("lat", "Latitude", tk.CENTER, tk.CENTER, tk.NO),
            LocationsView.__Column("long", "Longitude", tk.CENTER, tk.CENTER, tk.NO),
            LocationsView.__Column("tz", "Timezone", tk.W, tk.W, tk.YES),
        ]
        ids = [column.iid for column in self._columns]

        # create the locations view without the tree label column being visible
        scrollbar = tk.Scrollbar(self)
        scrollbar.grid(row=0, column=1, sticky=tk.N + tk.S + tk.E)
        self._tree = ttk.Treeview(self, columns=ids, yscrollcommand=scrollbar.set, padding=(4, 2), show='headings',
                                  selectmode=tk.EXTENDED if multi_select else tk.BROWSE)
        self._tree.grid(row=0, column=0, sticky=tk.NSEW)
        scrollbar.config(command=self._tree.yview)

        # setup each of the columns
        for idx, column in enumerate(self._columns):
            self._tree.heading(column.iid, text=column.text, anchor=column.heading_anchor)
            self._tree.column(column.iid, stretch=column.stretch, anchor=column.column_anchor)

        # get the width of the heading labels
        measure = nametofont(Style().lookup(f'{self._tree.winfo_class()}.Heading', 'font')).measure
        pad = measure('0' * 2)
        self._heading_widths = [pad + measure(column.text) for column in self._columns]

        # check if the alias column should be hidden
        if hide_alias:
            ids.remove('alias')
            self._tree.configure(displaycolumns=ids)

    def refresh(self, locations: List[PyLocation]):

        # clear the tree
        for item in self._tree.get_children():
            self._tree.delete(item)

        # get the column widths
        column_widths = copy(self._heading_widths)
        measure = nametofont(Style().lookup(self._tree.winfo_class(), 'font')).measure
        pad = measure('0' * 2)
        for location in locations:
            column_widths[0] = max(column_widths[0], measure(location.name) + pad)
            column_widths[1] = max(column_widths[1], measure(location.alias) + pad)
            column_widths[2] = max(column_widths[2], measure(location.latitude) + pad)
            column_widths[3] = max(column_widths[3], measure(location.longitude) + pad)
            column_widths[4] = max(column_widths[4], measure(location.tz) + pad)

        # set the width of the view columns
        for column, width in enumerate(column_widths):
            self._tree.column(column, minwidth=width, width=width)

        # repopulate the tree
        for index, location in enumerate(locations):
            self._tree.insert('', 'end', iid=index, values=(
                location.name, location.alias, location.latitude, location.longitude, location.tz
            ))

        # set focus to the first item in the tree
        self._tree.focus_set()
        if locations:
            self._tree.selection_set(0)
            self._tree.focus(0)

    def bind_event(self, event: str, handler: Callable[[Event], None]):
        self._tree.bind(event, handler)

    def get_selection(self) -> List[int]:
        return [int(s) for s in self._tree.selection()]

    def set_selection(self, selection: List[int]):
        self._tree.selection_set(selection)

    def selection_at(self, x: int, y: int) -> Optional[int]:
        item = self._tree.identify('item', x, y)
        return int(item) if item else None

    def context_xy(self, event: Event) -> Tuple[int, int]:
        """A helper to get the selected row x,y coordinate for popup menus."""

        # the popup menu needs the screen coordinates close to a selection
        left, top = self.winfo_rootx(), self.winfo_rooty()
        default = left, top + 20

        # make sure something is selected
        selection = self.get_selection()
        if not selection:
            return default

        # this is common through the remaining code
        def selection_xy(bbox_):
            _, y_, _, height_ = bbox_
            return left + event.x, top + y_ + round((float(height_) * .75))

        # retrieve the bbox of the left hand column
        iid_bbox = lambda iid_: self._tree.bbox(iid_, column=0)

        if len(selection) == 1:
            # make sure the selection is visible
            bbox = iid_bbox(selection[0])
            return selection_xy(bbox) if bbox else default

        # see if the mouse is over a selection
        mouse_iid = self.selection_at(event.x, event.y)
        if mouse_iid in selection:
            bbox = iid_bbox(mouse_iid)
            return selection_xy(bbox)

        # make sure the selection is in order
        selection.sort()

        # find the first visible selection
        first_iid = None
        first_bbox = None
        for iid in selection:
            first_bbox = iid_bbox(iid)
            if first_bbox:
                first_iid = iid
                break

        # find the last visible selection
        last_iid = None
        last_bbox = None
        for iid in reversed(selection):
            last_bbox = iid_bbox(iid)
            if last_bbox:
                last_iid = iid
                break

        # make sure the selections are visible
        if not first_bbox and not last_bbox:
            return default
        if not first_bbox:
            return selection_xy(last_bbox)
        if not last_bbox:
            return selection_xy(first_bbox)

        # if the mouse is somewhere between use the event position
        if first_iid <= mouse_iid <= last_iid:
            return left + event.x, top + event.y

        return selection_xy(last_bbox) if mouse_iid > last_iid else selection_xy(first_bbox)
