import tkinter as tk
import tkinter.messagebox as messagebox
import tkinter.ttk as ttk
from copy import copy
from datetime import date
from enum import IntEnum
from tkinter import IntVar
from tkinter.simpledialog import Dialog
from typing import Callable, List, Optional

from dateutil.relativedelta import relativedelta
from py_weather_lib import PyDateRange, PyHistoryDates

from .graphs import ConditionsType, PrecipitationType, TemperaturesType
from ..widgets import DateRangeSelector
from ...config import get_logger

__all__ = ['GraphSelector', 'GraphSelection', 'GraphType']
log = get_logger(__name__)


class GraphType(IntEnum):
    UNKNOWN = 0,
    TEMPERATURE = 1,
    PRECIPITATION = 2,
    CONDITIONS = 3


class GraphSelection:
    """A data class that holds the current graph selection."""

    def __init__(self, date_range: PyDateRange, primary_location=0, graph_type=GraphType.UNKNOWN,
                 temperature_type=TemperaturesType.UNKNOWN, precipitation_type=PrecipitationType.UNKNOWN,
                 condition_type=ConditionsType.UNKNOWN):
        self.is_ok = False
        self.primary_location = max(0, primary_location)
        self.graph_type = graph_type
        self.date_range = date_range
        self.graph_type = graph_type if graph_type != GraphType.UNKNOWN else GraphType.TEMPERATURE
        self.temperature_type = temperature_type if temperature_type != TemperaturesType.UNKNOWN else TemperaturesType.HIGH
        self.precipitation_type = precipitation_type if precipitation_type != PrecipitationType.UNKNOWN else PrecipitationType.RAIN
        self.conditions_type = condition_type if condition_type != ConditionsType.UNKNOWN else ConditionsType.WIND_SPEED

    def __bool__(self):
        return self.is_ok


class GraphSelector(Dialog):
    """The dialog the facilitates selection of a history graph."""

    def __init__(self, parent, graph_selection: GraphSelection, history_dates: List[PyHistoryDates]):
        graph_selection.is_ok = False
        self._graph_selection = graph_selection

        # make sure there are dates to graph
        self._locations_history_dates = []
        locations_wo_histories = []
        for location_history_dates in history_dates:
            if not location_history_dates.history_dates:
                locations_wo_histories.append(location_history_dates.location.name)
                continue
            if len(location_history_dates.history_dates) == 1:
                date_range = location_history_dates.history_dates[0]
                if date_range.start == date_range.end:
                    locations_wo_histories.append(location_history_dates.location.name)
                    continue
            self._locations_history_dates.append(location_history_dates)
        if locations_wo_histories:
            log.warning(f'The following locations did not have histories: {", ".join(locations_wo_histories)}')
        if not self._locations_history_dates:
            messagebox.showwarning('Graph Selector', 'There are no histories to graph.')
            return

        # default the history dates to the primary location
        self._history_dates = self._locations_history_dates[self._graph_selection.primary_location].history_dates
        self._locations_combobox: Optional[ttk.Combobox] = None

        # you need to remember the current date selection incase the choice is oob
        self._date_range_selector: Optional[DateRangeSelector] = None
        self._selector_tabs: Optional[SelectorTabs] = None
        super().__init__(parent, title='History Graph Selector')

    def body(self, parent: tk.Frame) -> tk.Widget:
        """Add the graph selection fields to the Dialog provided frame."""
        primary_location_name = self._locations_history_dates[self._graph_selection.primary_location].location.name
        # the primary location selector
        if len(self._locations_history_dates) > 1:
            locations_frame = tk.LabelFrame(parent, text='Locations', labelanchor=tk.N, padx=5, pady=2)
            locations_frame.rowconfigure(0, weight=1)
            locations_frame.columnconfigure(0, weight=1)
            locations_frame.columnconfigure(1, weight=1)
            locations_frame.grid(row=0, sticky=tk.NSEW)
            tk.Label(locations_frame, text='Primary:').grid(row=0, column=0, sticky=tk.E, padx=1, pady=5)
            location_names = [lhd.location.name for lhd in self._locations_history_dates]
            max_width = max([len(name) for name in location_names])
            self._locations_combobox = ttk.Combobox(locations_frame, values=location_names, width=max_width,
                                                    height=5, state='readonly')
            self._locations_combobox.grid(row=0, column=1, sticky=tk.W, padx=1, pady=5)
            self._locations_combobox.set(primary_location_name)
            self._locations_combobox.bind('<<ComboboxSelected>>', self._primary_location_selected)
        else:
            tk.Label(parent, text=f'{primary_location_name} Graph Selection').grid(row=0, column=0, padx=5, pady=5,
                                                                                   sticky=tk.E + tk.W + tk.S)
        # report calendar selection
        dates = tk.LabelFrame(parent, text='History Dates', labelanchor=tk.N, padx=5, pady=2)
        dates.grid(row=1, sticky=tk.NSEW)
        self._date_range_selector = DateRangeSelector(dates, self._history_dates,
                                                      copy(self._graph_selection.date_range))

        # the graph selector tabs
        selectors_frame = tk.LabelFrame(parent, text='History Graphs Selection', padx=5, pady=2)
        self._selector_tabs = SelectorTabs(selectors_frame, self._graph_selection)
        self._selector_tabs.grid()
        selectors_frame.grid(sticky=tk.NSEW, pady=2, padx=2)
        return self._date_range_selector.initial_focus()

    def validate(self) -> bool:
        return self._selector_tabs.validate()

    def apply(self, event=None):
        self._graph_selection.date_range = self._date_range_selector.date_range()
        # self._graph_selection.primary_location = self._locations_combobox.current()
        self._graph_selection.primary_location = self._locations_combobox.current() if self._locations_combobox else 0
        self._selector_tabs.apply()
        self._graph_selection.is_ok = True

    def _primary_location_selected(self, _):
        location_history_dates = self._locations_history_dates[self._locations_combobox.current()]
        if location_history_dates.history_dates:
            date_range = location_history_dates.history_dates[-1]
        else:
            end = date.today()
            start = end - relativedelta(month=1)
            date_range = PyDateRange(start=start, end=end)
        self._date_range_selector.set_history_dates(location_history_dates.history_dates, date_range)


class SelectorTabs(ttk.Notebook):
    def __init__(self, parent, graph_selection: GraphSelection):
        self._graph_selection = graph_selection

        # create the notebook with a lhs set of tabs
        style = ttk.Style(parent)
        style.configure('lhs.TNotebook', tabposition='wn')
        style.configure('lhs.TNotebook', borderwidth=5)
        ttk.Notebook.__init__(self, parent, style='lhs.TNotebook')

        # the graph selectors
        self._tab_validators: List[Callable[[], bool]] = []
        tab_graph_types: List[GraphType] = []

        self._temperatures_frame = TemperaturesTab(self, self._graph_selection.temperature_type)
        self.add(self._temperatures_frame, text='Temperatures')
        tab_graph_types.append(GraphType.TEMPERATURE)
        self._tab_validators.append(self._temperatures_frame.validate)

        self._precipitation_frame = PrecipitationTab(self, self._graph_selection.precipitation_type)
        self.add(self._precipitation_frame, text='Precipitation')
        tab_graph_types.append(GraphType.PRECIPITATION)
        self._tab_validators.append(self._precipitation_frame.validate)

        self._conditions_frame = ConditionsTab(self, self._graph_selection.conditions_type)
        self.add(self._conditions_frame, text='Conditions')
        tab_graph_types.append(GraphType.CONDITIONS)
        self._tab_validators.append(self._conditions_frame.validate)

        # make sure the tab with the current graph type is selected
        self._graph_type = self._graph_selection.graph_type
        for index, graph_type in enumerate(tab_graph_types):
            if graph_type == self._graph_type:
                self.select(index)

        # update the current graph type when the tab changes
        def tab_changed(_):
            self._graph_type = tab_graph_types[self.index(self.select())]
            # self._graph_type = tab_graph_types[notebook.index(notebook.select())]

        self.bind('<<NotebookTabChanged>>', tab_changed)

    def apply(self):
        self._graph_selection.graph_type = self._graph_type
        self._graph_selection.temperature_type = self._temperatures_frame.temperature_type()
        self._graph_selection.precipitation_type = self._precipitation_frame.precipitation_type()
        self._graph_selection.conditions_type = self._conditions_frame.conditions_type()

    def validate(self) -> bool:
        for validator in self._tab_validators:
            if not validator():
                return False
        return True


class TemperaturesTab(tk.Frame):

    def __init__(self, parent, temperatures_type: TemperaturesType, **kwargs):
        tk.Frame.__init__(self, parent, **kwargs)
        self.rowconfigure(0, weight=1)
        self.columnconfigure(0, weight=1)
        self.columnconfigure(1, weight=1)
        self.columnconfigure(2, weight=1)
        #
        check_button = lambda t, v, c: tk.Checkbutton(self, text=t, variable=v, command=c)

        # the high checkbox
        def high_clicked():
            if self.temperature_type() == TemperaturesType.UNKNOWN:
                self._high.set(1)

        self._high = IntVar(self, value=bool(temperatures_type & TemperaturesType.HIGH))
        check_button('High Temperature', self._high, high_clicked).grid(row=0, column=0, padx=2)

        # the low checkbox
        def low_clicked():
            if self.temperature_type() == TemperaturesType.UNKNOWN:
                self._low.set(1)

        self._low = IntVar(self, value=bool(temperatures_type & TemperaturesType.LOW))
        check_button('Low Temperature', self._low, low_clicked).grid(row=0, column=1, padx=2)

        # the mean checkbox
        def mean_clicked():
            if self.temperature_type() == TemperaturesType.UNKNOWN:
                self._mean.set(1)

        self._mean = IntVar(self, value=bool(temperatures_type & TemperaturesType.MEAN))
        check_button('Mean Temperature', self._mean, mean_clicked).grid(row=0, column=2, padx=2)

    def validate(self) -> bool:
        if self.temperature_type() == TemperaturesType.UNKNOWN:
            messagebox.showwarning('Temperatures Selection', 'Select a temperature graph.')
            return False
        return True

    def temperature_type(self) -> TemperaturesType:
        value = 0
        if self._high.get():
            value += TemperaturesType.HIGH.value
        if self._low.get():
            value += TemperaturesType.LOW.value
        if self._mean.get():
            value += TemperaturesType.MEAN.value
        return TemperaturesType(value)


class PrecipitationTab(tk.Frame):

    def __init__(self, parent, precipitation_type: PrecipitationType, **kwargs):
        tk.Frame.__init__(self, parent, **kwargs)
        self.rowconfigure(0, weight=1)
        self.columnconfigure(0, weight=1)
        self.columnconfigure(1, weight=1)
        self.columnconfigure(2, weight=1)

        check_button = lambda t, v, c: tk.Checkbutton(self, text=t, variable=v, command=c)

        self._rain = IntVar(self, value=precipitation_type == PrecipitationType.RAIN)
        check_button('Rain Amount (in)', self._rain, self._rain_clicked).grid(row=0, column=0, padx=2)

        self._humidity = IntVar(self, value=precipitation_type == PrecipitationType.HUMIDITY)
        check_button('Percent Humidity', self._humidity, self._humidity_clicked).grid(row=0, column=1, padx=2)

        self._cloud_cover = IntVar(self, value=precipitation_type == PrecipitationType.CLOUD_COVER)
        check_button('Cloud Cover', self._cloud_cover, self._cloud_cover_clicked).grid(row=0, column=2, padx=2)

    def _rain_clicked(self):
        if self.precipitation_type() == PrecipitationType.UNKNOWN:
            self._rain.set(1)
        self._humidity.set(0)
        self._cloud_cover.set(0)

    def _humidity_clicked(self):
        if self.precipitation_type() == PrecipitationType.UNKNOWN:
            self._humidity.set(1)
        self._rain.set(0)
        self._cloud_cover.set(0)

    def _cloud_cover_clicked(self):
        if self.precipitation_type() == PrecipitationType.UNKNOWN:
            self._cloud_cover.set(1)
        self._rain.set(0)
        self._humidity.set(0)

    def validate(self) -> bool:
        if self.precipitation_type() == PrecipitationType.UNKNOWN:
            messagebox.showwarning('Precipitation Selection', 'Select a precipitation graph.')
            return False
        return True

    def precipitation_type(self) -> PrecipitationType:
        if self._rain.get():
            return PrecipitationType.RAIN
        if self._humidity.get():
            return PrecipitationType.HUMIDITY
        if self._cloud_cover.get():
            return PrecipitationType.CLOUD_COVER
        return PrecipitationType.UNKNOWN


class ConditionsTab(tk.Frame):

    def __init__(self, parent, conditions_type: ConditionsType, **kwargs):
        tk.Frame.__init__(self, parent, **kwargs)
        self.rowconfigure(0, weight=1)
        self.columnconfigure(0, weight=1)
        self.columnconfigure(1, weight=1)
        self.columnconfigure(2, weight=1)

        check_button = lambda t, v, c: tk.Checkbutton(self, text=t, variable=v, command=c)

        self._wind_speed = IntVar(self, value=conditions_type & ConditionsType.WIND_SPEED == ConditionsType.WIND_SPEED)
        check_button('Wind Speed', self._wind_speed, self._wind_speed_clicked).grid(row=0, column=0, padx=2)

        self._wind_gust = IntVar(self, value=conditions_type & ConditionsType.WIND_GUST == ConditionsType.WIND_GUST)
        check_button('Wind Gusts', self._wind_gust, self._wind_gust_clicked).grid(row=0, column=1, padx=2)

        self._uv_index = IntVar(self, value=conditions_type & ConditionsType.UV_INDEX == ConditionsType.UV_INDEX)
        check_button('UV Index', self._uv_index, self._uv_index_clicked).grid(row=0, column=2, padx=2)

    def _wind_speed_clicked(self):
        if self.conditions_type() == ConditionsType.UNKNOWN:
            self._wind_speed.set(1)
        self._uv_index.set(0)

    def _wind_gust_clicked(self):
        if self.conditions_type() == ConditionsType.UNKNOWN:
            self._wind_gust.set(1)
        self._uv_index.set(0)

    def _uv_index_clicked(self):
        if self.conditions_type() == ConditionsType.UNKNOWN:
            self._uv_index.set(1)
        self._wind_speed.set(0)
        self._wind_gust.set(0)

    def validate(self) -> bool:
        if self.conditions_type() == ConditionsType.UNKNOWN:
            messagebox.showwarning('Conditions Selection', 'Select a conditions graph.')
            return False
        return True

    def conditions_type(self) -> ConditionsType:
        value = 0
        if self._wind_speed.get():
            value += ConditionsType.WIND_SPEED
        if self._wind_gust.get():
            value += ConditionsType.WIND_GUST
        if self._uv_index.get():
            value += ConditionsType.UV_INDEX
        return ConditionsType(value)
