import tkinter as tk
import tkinter.messagebox as messagebox
from datetime import UTC, datetime
from tkinter.font import nametofont
from tkinter.simpledialog import Dialog
from typing import List, Optional
from zoneinfo import ZoneInfo

from py_weather_lib import (PyDailyHistories, PyDateRange, PyHistoryDates, PyLocationFilter, PyLocationFilters)

from .infrastructure import Stopwatch, WeatherView
from .widgets import (CellStyle, Column, DateRangeSelector, Pad, Row, Sheet)
from ..config import get_logger
from ..domain import WeatherData

__all__ = ['HistoryReport']
log = get_logger(__name__)


class HistoryReport(WeatherView):
    def __init__(self, parent, location_alias: str, weather_data: WeatherData,
                 date_range: Optional[PyDateRange] = None):
        self._location_alias = location_alias
        self._weather_data = weather_data
        # get the report selection
        stopwatch = Stopwatch()
        # make sure there are histories to report
        filters = PyLocationFilters([PyLocationFilter(name=self._location_alias)])
        location_history_dates = self._weather_data.backend.get_history_dates(filters)[0]
        history_dates_t = str(stopwatch)
        self._is_cancelled = True
        if not location_history_dates.history_dates:
            messagebox.showinfo(title='Report History',
                                message=f'{location_history_dates.location.name} does not have weather data history.')
            return
        # use the latest history dates as a default
        if not date_range:
            date_range = location_history_dates.history_dates[-1]
        report_selection = ReportSelection(date_range, full=True)
        ReportSelector(parent, location_history_dates, report_selection)
        if not report_selection:
            self._sheet = None
        else:
            stopwatch.restart()
            filters = PyLocationFilter(name=self._location_alias)
            daily_histories = self._weather_data.backend.get_daily_history(filters, report_selection.date_range)
            headings = _create_headings(report_selection)
            contents = _create_contents(daily_histories, report_selection)
            self._sheet = Sheet(parent, headings, contents, view_labeled=True)
            log.info('get_histories: %s get_daily_histories: %s', history_dates_t, str(stopwatch))
            self._is_cancelled = False

    def __bool__(self):
        return not self._is_cancelled

    def view(self):
        """Return the history report view if available otherwise raise an AttributeError."""
        if not self._sheet:
            raise AttributeError('History report was cancelled.')
        return self._sheet


class ReportSelection:
    def __init__(self, date_range: PyDateRange, temperatures=False, conditions=False, precipitation=False,
                 summary=False,
                 full=False):
        self.date_range = date_range
        # initialize the categories
        if full:
            temperatures = conditions = precipitation = summary = True
        elif not (temperatures or conditions or precipitation or summary):
            temperatures = True
        self.temperatures = temperatures
        self.conditions = conditions
        self.precipitation = precipitation
        self.summary = summary
        self.ok = False

    def __bool__(self):
        return self.ok

    def __str__(self):
        categories = []
        if self.temperatures:
            categories.append('Temperatures')
        if self.precipitation:
            categories.append('Precipitation')
        if self.conditions:
            categories.append('Conditions')
        if self.summary:
            categories.append('Summary')
        return f'{self.date_range.start} to {self.date_range.end} [{",".join(categories)}]'


class ReportSelector(Dialog):
    def __init__(self, parent, location_history_dates: PyHistoryDates, report_selection: ReportSelection):
        self._location_history_dates = location_history_dates
        report_selection.ok = False
        self._report_selection = report_selection
        self._date_range_selector: Optional[DateRangeSelector] = None
        self._temperatures = tk.IntVar(parent, report_selection.temperatures)
        self._precipitation = tk.IntVar(parent, report_selection.precipitation)
        self._conditions = tk.IntVar(parent, report_selection.conditions)
        self._summary = tk.IntVar(parent, report_selection.summary)
        super().__init__(parent, title='Report History Criteria')

    def body(self, parent: tk.Frame) -> tk.Widget:
        """Add the dialog fields to the frame supplied by the Dialog."""

        # report calendar selection
        dates = tk.LabelFrame(parent, text='History Dates', labelanchor=tk.N, padx=5, pady=2)
        dates.grid(row=0, sticky=tk.NSEW)
        self._date_range_selector = DateRangeSelector(dates, self._location_history_dates.history_dates,
                                                      date_range=self._report_selection.date_range)

        # the report categories
        categories = tk.LabelFrame(parent, text='Categories', labelanchor=tk.N, padx=5, pady=5)
        categories.grid(row=1, sticky=tk.N + tk.S)
        tk.Checkbutton(categories, text='Temperatures', variable=self._temperatures).grid(row=0, column=0, sticky=tk.W)
        tk.Checkbutton(categories, text='Precipitation', variable=self._precipitation).grid(row=0, column=1,
                                                                                            sticky=tk.W)
        tk.Checkbutton(categories, text='Conditions', variable=self._conditions).grid(row=0, column=2, sticky=tk.W)
        tk.Checkbutton(categories, text='Summary', variable=self._summary).grid(row=0, column=3, sticky=tk.W)

        return self._date_range_selector.initial_focus()

    def apply(self, event=None):
        self._report_selection.date_range = self._date_range_selector.date_range()
        self._report_selection.temperatures = self._temperatures.get()
        self._report_selection.precipitation = self._precipitation.get()
        self._report_selection.conditions = self._conditions.get()
        self._report_selection.summary = self._summary.get()
        self._report_selection.ok = True


def _create_headings(report_selection: ReportSelection) -> List[Row]:
    font = nametofont('TkHeadingFont')
    font.configure(weight='bold')
    style = CellStyle(font, outlined=True)
    pad = Pad(surround=2)
    headings = []
    # first heading row
    columns = [Column.header('Date', pad, style, justify=tk.CENTER, label=True, row_span=2)]
    if report_selection.temperatures:
        columns += [
            Column.header('Temperature', pad, style, justify=tk.CENTER, column_span=3),
            Column.span(),
            Column.span(),
            Column.header('Dew\nPoint', pad, style, justify=tk.CENTER, row_span=2)
        ]
    if report_selection.precipitation:
        columns += [
            Column.header('Cloud\nCover', pad, style, justify=tk.CENTER, row_span=2),
            Column.header('Humidity', pad, style, justify=tk.CENTER, row_span=2),
            Column.header('Precipitation', pad, style, justify=tk.CENTER, column_span=3),
            Column.span(),
            Column.span(),
        ]
    if report_selection.conditions:
        columns += [
            Column.header('Wind', pad, style, justify=tk.CENTER, column_span=3),
            Column.span(),
            Column.span(),
            Column.header('Pressure', pad, style, justify=tk.CENTER, row_span=2),
            Column.header('UV\nIndex', pad, style, justify=tk.CENTER, row_span=2),
        ]
    if report_selection.summary:
        columns += [
            Column.header('Sunrise', pad, style, justify=tk.CENTER, row_span=2),
            Column.header('Sunset', pad, style, justify=tk.CENTER, row_span=2),
            Column.header('Daylight\nHours', pad, style, justify=tk.CENTER, row_span=2),
            Column.header('Moon\nPhase', pad, style, justify=tk.CENTER, row_span=2),
            Column.header('Summary', pad, style, row_span=2),
        ]
    headings.append(Row(0, columns, is_header=True))
    columns = [Column.span()]
    if report_selection.temperatures:
        columns += [
            Column.header('High', pad, style, justify=tk.CENTER),
            Column.header('Low', pad, style, justify=tk.CENTER),
            Column.header('Mean', pad, style, justify=tk.CENTER),
            Column.span(),
        ]
    if report_selection.precipitation:
        columns += [
            Column.span(),
            Column.span(),
            Column.header('Chance', pad, style, justify=tk.CENTER),
            Column.header('Amount', pad, style, justify=tk.CENTER),
            Column.header('Type', pad, style, justify=tk.CENTER),
        ]
    if report_selection.conditions:
        columns += [
            Column.header('Speed', pad, style, justify=tk.CENTER),
            Column.header('Gust', pad, style, justify=tk.CENTER),
            Column.header('Bearing', pad, style, justify=tk.CENTER),
            Column.span(),
            Column.span(),
        ]
    if report_selection.summary:
        columns += [
            Column.span(),
            Column.span(),
            Column.span(),
            Column.span(),
            Column.span(),
        ]
    headings.append(Row(1, columns, is_header=True))
    return headings


def _create_contents(daily_histories: PyDailyHistories, report_selection: ReportSelection) -> List[Row]:
    tz = ZoneInfo(daily_histories.location.tz)
    text_style = CellStyle(nametofont('TkTextFont'), outlined=True)
    number_style = CellStyle(nametofont('TkFixedFont'), outlined=True)
    default_pad = Pad(surround=1)
    right_pad = Pad(left=1, top=1, right=2, bottom=1)
    to_int = lambda v, w: '' if not v else f'{round(v):{w}}'
    to_float = lambda f, w: '' if not f else f'{f:{w}.1f}'
    to_percent = lambda p, w: '' if not p else f'{round(p * 100):{w}}%'
    to_hhmm = lambda dt: dt.replace(tzinfo=UTC).astimezone(tz).strftime('%H:%M')
    to_date = lambda d: d.strftime('%b %d, %Y')
    contents = []
    for row_index, history in enumerate(daily_histories.histories):
        columns = [Column.content(to_date(history.date), default_pad, text_style, label=True, justify=tk.RIGHT)]
        if report_selection.temperatures:
            columns += [
                Column.content(to_int(history.temperature_high, 4), right_pad, number_style, justify=tk.RIGHT),
                Column.content(to_int(history.temperature_low, 4), right_pad, number_style, justify=tk.RIGHT),
                Column.content(to_int(history.temperature_mean, 4), right_pad, number_style, justify=tk.RIGHT),
                Column.content(to_int(history.dew_point, 4), right_pad, number_style, justify=tk.RIGHT),
            ]
        if report_selection.precipitation:
            columns += [
                Column.content(to_percent(history.cloud_cover, 4), right_pad, number_style, justify=tk.RIGHT),
                Column.content(to_percent(history.humidity, 4), right_pad, number_style, justify=tk.RIGHT),
                Column.content(to_percent(history.precipitation_chance, 4), right_pad, number_style, justify=tk.RIGHT),
                Column.content(to_float(history.precipitation_amount, 4), right_pad, number_style, justify=tk.RIGHT),
                Column.content(history.precipitation_type, default_pad, text_style, justify=tk.CENTER),
            ]
        if report_selection.conditions:
            columns += [
                Column.content(to_float(history.wind_speed, 5), right_pad, number_style, justify=tk.RIGHT),
                Column.content(to_float(history.wind_gust, 5), right_pad, number_style, justify=tk.RIGHT),
                Column.content(__wind_bearing(history.wind_direction), default_pad, text_style, justify=tk.CENTER),
                Column.content(to_float(history.pressure, 6), right_pad, number_style, justify=tk.RIGHT),
                Column.content(__uv_index(history.uv_index), default_pad, text_style, justify=tk.CENTER),
            ]
        if report_selection.summary:
            columns += [
                Column.content(to_hhmm(history.sunrise), default_pad, number_style, justify=tk.CENTER),
                Column.content(to_hhmm(history.sunset), default_pad, number_style, justify=tk.CENTER),
                Column.content(__daylight_hours(history.sunrise, history.sunset), default_pad, number_style,
                               justify=tk.RIGHT),
                Column.content(__moon_phase(history.moon_phase), default_pad, text_style, justify=tk.CENTER),
                Column.content(history.description, default_pad, text_style),
            ]
        contents.append(Row(row_index, columns))
    return contents


__bearings = ['N', 'NNE', 'NE', 'ENE', 'E', 'ESE', 'SE', 'SSE', 'S', 'SSW', 'SW', 'WSW', 'W', 'WNW', 'NW', 'NNW']


def __wind_bearing(bearing: Optional[float] = None) -> str | None:
    if bearing is not None:
        return __bearings[round(bearing / 22.5) % 16]
    return None


def __uv_index(index: Optional[float] = None) -> str | None:
    if index is not None:
        index = int(index)
        if 0 <= index <= 2:
            return 'low'
        if 3 <= index <= 5:
            return 'moderate'
        if 6 <= index <= 7:
            return 'high'
        if 8 <= index <= 10:
            return 'very high'
        if index > 10:
            return 'extreme'
    return None


def __moon_phase(phase: Optional[float] = None) -> str | None:
    if phase is not None:
        if 0.0 <= phase <= 0.01:
            return 'new moon'
        if 0.01 < phase < 0.24:
            return 'waxing crescent'
        if 0.24 <= phase <= 0.26:
            return 'first quarter'
        if 0.26 < phase < 0.49:
            return 'waxing gibbous'
        if 0.49 <= phase <= 0.51:
            return 'full moon'
        if 0.51 < phase < 0.74:
            return 'waning gibbous'
        if 0.74 <= phase <= 0.76:
            return 'last quarter'
        if 0.76 < phase <= 1.0:
            return 'waning crescent'
        return '?'
    return None


def __daylight_hours(sunrise: datetime, sunset: datetime) -> str:
    seconds = (sunset - sunrise).total_seconds()
    hours, remainder = divmod(seconds, 3600)
    minutes, seconds = divmod(remainder, 60)
    return f'{int(hours):02}:{int(minutes):02}:{int(seconds):02}'
