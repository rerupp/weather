import tkinter as tk
import tkinter.messagebox as messagebox
import tkinter.ttk as ttk
from datetime import date
from tkinter.simpledialog import Dialog
from typing import List, Optional

from dateutil.relativedelta import relativedelta
from py_weather_lib import PyDateRange, PyHistoryClient, PyLocation, PyLocationFilter, PyLocationFilters

from .widgets import DateRangeSelector
from ..config import get_logger
from ..domain import WeatherData

__all__ = ['AddHistory']
log = get_logger(__name__)


def _info(msg: str) -> None:
    messagebox.showinfo(title='Add History', message=msg)


def _warn(msg: str) -> None:
    messagebox.showwarning(title='Add History', message=msg)


def _error(msg: str, err: SystemError) -> None:
    log.error('%s:\n%s', msg, err)
    messagebox.showerror('Add History', f'{msg}\nSee the log file for more information.')


class AddHistory:
    def __init__(self, parent, location: PyLocation, weather_data: WeatherData):
        self._parent = parent
        self._location = location
        self._weather_data = weather_data
        self._is_history_added = False
        # get the locations existing history dates
        try:
            one_month = relativedelta(months=1)
            one_day = relativedelta(days=1)
            filters = PyLocationFilters([PyLocationFilter(name=self._location.alias)])
            history_dates = self._weather_data.backend.get_history_dates(filters)[0].history_dates
            today = date.today()
            if not history_dates:
                # when there are no history dates default to the last month
                start = today - one_month
                end = today
            elif history_dates[-1].end == today:
                # if the location has the latest weather history try to get the prior month of history
                end = history_dates[-1].start - one_day
                start = end - one_month
                # make sure the start date isn't covered by the previous history
                if len(history_dates) > 1 and history_dates[-2].end > start:
                    start = history_dates[-2].end + one_day
            else:
                end = today
                start = end - one_month
                if history_dates[-1].end > start:
                    start = history_dates[-1].end + one_day
            self._date_range = PyDateRange(start, end)
        except SystemError as err:
            _error(f'Error getting the history dates for {location.name}.', err)
            return
        # if not DateSelector(self._parent, self._location, self._date_range, history_dates).is_canceled:
        date_selector = DateSelector(self._parent, self._location, self._date_range, history_dates)
        if not date_selector.is_canceled():
            self._date_range = date_selector.date_range()
            self._get_history()

    @property
    def is_history_added(self):
        return self._is_history_added

    def _get_history(self):
        try:
            history_client = self._weather_data.backend.get_history_client()
            history_client.execute(location=self._location, date_range=self._date_range)
            get_history = GetHistory(self._parent, self._location, history_client)
            if not get_history.is_canceled:
                # the history client won't know there was a server error until you try and get the response
                daily_histories = history_client.get()
                self._weather_data.backend.add_histories(daily_histories)
                history_count = len(daily_histories.histories)
                _info(f'{history_count} histories were added to {self._location.name}.')
                self._is_history_added = True
        except SystemError as err:
            _error(f'Error getting history data for {self._location.name}.', err)


class DateSelector(Dialog):
    def __init__(self, parent, location: PyLocation, date_range: PyDateRange, history_dates: List[PyDateRange]):
        self._location = location
        self._date_range = date_range
        self._history_dates = history_dates
        self._date_range_selector: Optional[DateRangeSelector] = None
        self._is_canceled = True
        super().__init__(parent, title='Add History')

    def is_canceled(self):
        return self._is_canceled

    def date_range(self) -> PyDateRange:
        return self._date_range

    def body(self, parent: tk.Frame) -> tk.Widget:
        dates = tk.LabelFrame(parent, text='History Dates', labelanchor=tk.N, padx=5, pady=2)
        dates.grid(row=0, sticky=tk.NSEW)
        self._date_range_selector = DateRangeSelector(dates, [], self._date_range, history_selector=False)
        return self._date_range_selector.initial_focus()

    def validate(self) -> bool:
        date_range = self._date_range_selector.date_range()
        # check if the dates are already part of the history
        for dr in self._history_dates:
            if dr.contains(date_range.start):
                _warn('History already exists for the starting date.')
                return False
            if dr.contains(date_range.end):
                _warn('History already exists for the ending date.')
                return False
            if date_range.start < dr.start and date_range.end > dr.end:
                fmt = lambda d: d.strftime('%b-%d-%Y')
                if dr.start == dr.end:
                    existing_date_range = fmt(dr.start)
                else:
                    existing_date_range = f'{fmt(dr.start)} thru {fmt(dr.end)}'
                _warn(f'The selected dates cover existing histories ({existing_date_range}).')
                return False
        return True

    def apply(self):
        self._is_canceled = False
        self._date_range = self._date_range_selector.date_range()


class GetHistory(Dialog):
    def __init__(self, parent, location: PyLocation, history_client: PyHistoryClient):
        self._history_client = history_client
        self._location = location
        self._callback_id: Optional[str] = None
        self._progress_step: Optional[tk.DoubleVar] = None
        self._is_canceled = True
        super().__init__(parent, title=f'Getting {location.name} History')

    @property
    def is_canceled(self):
        return self._is_canceled

    def body(self, parent: tk.Frame):
        self._progress_step = tk.DoubleVar(parent)
        maximum = 100
        progress_bar = ttk.Progressbar(parent, orient=tk.HORIZONTAL, variable=self._progress_step, maximum=maximum,
                                       mode='determinate', length=200)
        progress_bar.grid()
        direction = tk.E

        def callback():
            nonlocal direction
            # check if the client received the response
            if self._history_client.poll():
                self._is_canceled = False
                self._callback_id = None
                self.ok()
                return
            progress_step = self._progress_step.get()
            if direction == tk.E:
                if progress_step >= maximum:
                    direction = tk.W
                    progress_step -= 1
                else:
                    progress_step += 1
            else:
                if progress_step <= 0:
                    direction = tk.E
                    progress_step += 1
                else:
                    progress_step -= 1
            self._progress_step.set(progress_step)
            # noinspection PyTypeChecker
            self._callback_id = self.after(100, callback)

        callback()

    def buttonbox(self):
        box = tk.Frame(self)
        w = tk.Button(box, text="Cancel", width=10, command=self.cancel, default=tk.ACTIVE)
        w.pack(side=tk.LEFT, padx=5, pady=5)
        self.bind("<Escape>", self.cancel)
        box.pack()

    def cancel(self, event=None):
        if self._callback_id:
            self.after_cancel(self._callback_id)
        super().cancel(event)
