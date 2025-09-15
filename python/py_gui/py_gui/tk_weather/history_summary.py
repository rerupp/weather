import tkinter as tk
import tkinter.messagebox as messagebox
import tkinter.ttk as ttk
from collections import namedtuple
from copy import copy
from tkinter.font import nametofont
from typing import List

from py_weather_lib import PyHistorySummaries, PyLocationFilters

from .infrastructure import WeatherView
from ..config import get_logger
from ..domain import WeatherData

__all__ = ['HistorySummary']
log = get_logger(__name__)


class HistorySummary(WeatherView):
    def __init__(self, weather_data_: WeatherData, parent):
        super().__init__()
        self._weather_data = weather_data_
        self._view = SummaryView(parent)
        self.refresh()

    def view(self) -> tk.Frame:
        return self._view

    def refresh(self):
        try:
            content = self._weather_data.backend.get_history_summary(PyLocationFilters([]))
            self._view.refresh(content)
        except SystemError as error:
            msg = 'There was a problem getting the history summary.'
            log.error('%s\n%s', msg, error)
            messagebox.showerror('History Summary View', f'{msg}\nCheck the log for more information')


class SummaryView(tk.Frame):
    # keep the tree view columns to yourself
    __Column = namedtuple('Column', ['iid', 'text', 'heading_anchor', 'column_anchor', 'stretch'])

    def __init__(self, parent, **kwargs):
        super().__init__(parent, **kwargs)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)

        scrollbar = tk.Scrollbar(self)
        scrollbar.grid(row=0, column=1, sticky=tk.N + tk.S + tk.E)

        # the view columns
        self._columns = [
            SummaryView.__Column("#0", "Location", tk.CENTER, tk.W, tk.NO),
            SummaryView.__Column("overall_size", "Overall Size", tk.CENTER, tk.E, tk.NO),
            SummaryView.__Column("history_count", "History Count", tk.CENTER, tk.E, tk.NO),
            SummaryView.__Column("history_size", "History Size", tk.CENTER, tk.E, tk.NO),
            SummaryView.__Column("store_size", "Store Size", tk.CENTER, tk.E, tk.NO),
        ]

        ids = [cid.iid for cid in self._columns[1:]]
        self._tree = ttk.Treeview(self, columns=ids, yscrollcommand=scrollbar.set)
        self._tree.grid(row=0, column=0, sticky=tk.NSEW)

        scrollbar.config(command=self._tree.yview)

        heading_font = nametofont(ttk.Style().lookup(f'{self._tree.winfo_class()}.Heading', 'font'))
        get_width = heading_font.measure
        pad = get_width('0' * 2)
        self._heading_widths = [pad + get_width(column.text) for column in self._columns]

        for idx, column in enumerate(self._columns):
            self._tree.heading(column.iid, text=column.text, anchor=column.heading_anchor)
            self._tree.column(column.iid, stretch=column.stretch, anchor=column.column_anchor)

    def refresh(self, content: List[PyHistorySummaries]):
        tree = self._tree

        # clear the tree
        for item in tree.get_children():
            tree.delete(item)

        # create the report content
        Summary = namedtuple('Summary', ['name', 'overall_size', 'count', 'raw_size', 'store_size'])
        summaries: List[Summary] = []
        for history_summaries in content:
            summaries.append(Summary(
                name=history_summaries.location.name,
                overall_size=f'{history_summaries.overall_size / 1024:,.1f} KiB',
                count=f'{history_summaries.count:,}',
                raw_size=f'{history_summaries.raw_size / 1024:,.1f} KiB',
                store_size=f'{history_summaries.store_size / 1024:,.1f} KiB'
            ))

        # calculate the column widths
        measure = nametofont(ttk.Style().lookup(self._tree.winfo_class(), 'font')).measure

        # start with the heading widths
        column_widths = copy(self._heading_widths)

        pad = measure('0' * 4)
        # get the max column widths
        for summary in summaries:
            column_widths[0] = max(column_widths[0], measure(summary.name) + pad)
            column_widths[1] = max(column_widths[1], measure(summary.overall_size) + pad)
            column_widths[2] = max(column_widths[2], measure(summary.count) + pad)
            column_widths[3] = max(column_widths[3], measure(summary.raw_size) + pad)
            column_widths[4] = max(column_widths[4], measure(summary.store_size) + pad)

        # add in the width of the tree decorator
        tree.column('#0', minwidth=column_widths[0], width=column_widths[0] + 10)
        for column in range(1, len(column_widths)):
            tree_column = column - 1
            tree.column(tree_column, minwidth=column_widths[column], width=column_widths[column])

        # repopulate the tree
        for index, summary in enumerate(summaries):
            self._tree.insert('', 'end', iid=index, text=summary.name, values=(
                summary.overall_size, summary.count, summary.raw_size, summary.store_size
            ))

        # set focus to the first item in the tree
        self._tree.focus_set()
        self._tree.selection_set(0)
        self._tree.focus(0)

    def get_selection(self):
        return self._tree.selection()
