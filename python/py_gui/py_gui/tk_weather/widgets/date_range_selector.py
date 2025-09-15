import tkinter as tk
import tkinter.messagebox as messagebox
import tkinter.ttk as ttk
from copy import copy
from datetime import date
from typing import List, Optional

from tkcalendar import Calendar

from py_weather_lib import PyDateRange

__all__ = ['DateRangeSelector']


class DateRangeSelector:
    def __init__(self, parent, history_dates: List[PyDateRange], date_range: Optional[PyDateRange] = None,
                 history_selector=True):
        self._history_dates = copy(history_dates) if history_dates else []
        if date_range:
            self._current_date_range = copy(date_range)
        elif self._history_dates:
            self._current_date_range = copy(self._history_dates[-1])
        else:
            start = end = date.today()
            self._current_date_range = PyDateRange(start, end)

        def calendar(d: date) -> Calendar:
            (year, month, day) = d.timetuple()[0:3]
            return Calendar(parent, year=year, month=month, day=day, showweeknumbers=0, date_pattern='m/d/yyyy')

        s_sticky = tk.S + tk.E + tk.W
        n_sticky = tk.N + tk.E + tk.W
        tk.Label(parent, text='Starting').grid(row=0, column=0, sticky=s_sticky, pady=0)
        self._calendar_start = calendar(self._current_date_range.start)
        self._calendar_start.grid(row=1, column=0, sticky=n_sticky, pady=2, padx=2)
        self._calendar_start.bind('<<CalendarSelected>>', self._starting_date_selected)
        tk.Label(parent, text='Ending').grid(row=0, column=1, sticky=s_sticky)
        self._calendar_end = calendar(self._current_date_range.end)
        self._calendar_end.grid(row=1, column=1, sticky=n_sticky, pady=2, padx=2)
        self._calendar_end.bind('<<CalendarSelected>>', self._ending_date_selected)

        self._history_selector = history_selector
        if self._history_selector:
            # set up the available histories selector
            tk.Label(parent, text='Available Histories').grid(row=2, columnspan=2, sticky=tk.S)
            self._histories_combobox = ttk.Combobox(parent, width=28, height=5, state='readonly')
            self._histories_combobox.grid(row=3, columnspan=2, sticky=tk.N, pady=2)
            self._initialize_histories_combobox()
            self._histories_combobox.bind('<<ComboboxSelected>>', self._date_range_selected)

    def _initialize_histories_combobox(self):
        if not self._history_dates:
            self._histories_combobox.config(state='disabled')
        else:
            fmt = lambda d: d.strftime('%b-%d-%Y')

            def fmt_date_range(dr: PyDateRange) -> str:
                if dr.start == dr.end:
                    return fmt(dr.start)
                return f'{fmt(dr.start)} thru {fmt(dr.end)}'

            self._histories_combobox['values'] = [fmt_date_range(dr) for dr in self._history_dates]
            self._histories_combobox.set(fmt_date_range(self._current_date_range))

    def initial_focus(self) -> tk.Widget:
        return self._calendar_start

    def date_range(self) -> PyDateRange:
        return copy(self._current_date_range)

    def set_history_dates(self, history_dates: List[PyDateRange], selected_date_range: PyDateRange):
        if self._history_selector:
            self._history_dates = copy(history_dates)
            self._calendar_start.selection_set(selected_date_range.start)
            self._calendar_end.selection_set(selected_date_range.end)
            self._current_date_range = copy(selected_date_range)
            self._initialize_histories_combobox()

    def _date_range_selected(self, _):
        selected_date_range = self._history_dates[self._histories_combobox.current()]
        self._calendar_start.selection_set(selected_date_range.start)
        self._calendar_end.selection_set(selected_date_range.end)
        self._current_date_range = copy(selected_date_range)

    def _starting_date_selected(self, _):
        warn = lambda m: messagebox.showwarning('Start Date', m)
        start_date = self._calendar_start.selection_get()
        if start_date > date.today():
            warn('Start date is after today.')
            self._calendar_start.selection_set(self._current_date_range.start)
        elif start_date > self._calendar_end.selection_get():
            warn('Start date is after end date.')
            self._calendar_start.selection_set(self._current_date_range.start)
        elif not self._history_dates:
            self._current_date_range.start = start_date
        elif start_date > self._history_dates[-1].end:
            warn(f'Start date is after last history date.')
            self._calendar_start.selection_set(self._current_date_range.start)
        else:
            self._current_date_range.start = start_date

    def _ending_date_selected(self, _):
        warn = lambda m: messagebox.showwarning('End Date', m)
        end_date = self._calendar_end.selection_get()
        if end_date > date.today():
            warn('End date is after today.')
            self._calendar_end.selection_set(self._current_date_range.end)
        elif end_date < self._calendar_start.selection_get():
            warn('End date is prior to start date.')
            self._calendar_end.selection_set(self._current_date_range.end)
        elif not self._history_dates:
            self._current_date_range.end = end_date
        elif end_date < self._history_dates[0].start:
            warn('End date is prior to first history date.')
            self._calendar_end.selection_set(self._current_date_range.end)
        else:
            self._current_date_range.end = end_date
