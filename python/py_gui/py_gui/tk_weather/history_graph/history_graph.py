import tkinter as tk
from tkinter import messagebox
from typing import Callable, List, Optional, Union

from py_weather_lib import (
    PyDailyHistories, PyDateRange, PyHistoryDates, PyLocation, PyLocationFilter, PyLocationFilters
)

from .graph_selector import GraphSelection, GraphSelector, GraphType
from .graph_view import GraphView
from .graphs import Graphs
from ..infrastructure import Stopwatch, WeatherView
from ...config import get_logger
from ...domain import WeatherData

__all__ = ['HistoryGraph']
log = get_logger(__name__)


class HistoryGraph(WeatherView):

    @staticmethod
    def _warning(msg: str):
        messagebox.showwarning(title='History graph', message=msg)

    @staticmethod
    def _error(msg: str, err: SystemError):
        log.error('%s.\n%s', msg, err)
        messagebox.showerror(title='History graph', message=msg)

    def __init__(self, parent, locations: Union[PyLocation, List[PyLocation]], weather_data: WeatherData,
                 add_tab: Callable[[str, WeatherView], None], date_range: Optional[PyDateRange] = None):
        self._parent = parent
        self._weather_data = weather_data
        self._add_tab = add_tab

        # get the locations history dates
        if isinstance(locations, PyLocation):
            locations = [locations]

        self._locations_history_dates = self._get_locations_history_dates(locations)
        if not self._locations_history_dates:
            # HistoryGraph._warning('There are no history dates available for the locations.')
            return

        # always default to the first location in the history dates list
        initial_location = 0
        if not date_range:
            date_range = self._locations_history_dates[initial_location].history_dates[-1]
        self._graph_selection = GraphSelection(date_range, primary_location=initial_location)
        GraphSelector(self._parent, self._graph_selection, self._locations_history_dates)
        if not self._graph_selection:
            return

        self._graphs = Graphs()
        self._graph_view: Optional[GraphView] = None
        self._select_graph()

    def view(self) -> tk.Frame:
        return self._graph_view

    def _get_locations_history_dates(self, locations: List[PyLocation]) -> List[PyHistoryDates] | None:
        """
        Get the location history dates for the locations.
        """
        try:
            # filter out locations that do not have histories
            sw = Stopwatch()
            locations_without_histories = []
            locations_history_dates = []
            filters = [PyLocationFilter(name=l.alias) for l in locations] if locations else None
            for location_history_dates in self._weather_data.backend.get_history_dates(PyLocationFilters(filters)):
                if location_history_dates.history_dates:
                    locations_history_dates.append(location_history_dates)
                else:
                    locations_without_histories.append(location_history_dates.location.name)
            log.info('get locations history dates %s', sw)

            # log the locations that do not have histories
            if locations_without_histories:
                log.warning(f'The following locations do not have histories: {",".join(locations_without_histories)}')

            if not locations_history_dates:
                HistoryGraph._warning('There are no history dates available for the locations.')
            else:
                # the locations always have the primary first so make sure the returned location
                # history dates does the same
                if locations and len(locations_history_dates) > 1:
                    alias = locations[0].alias
                    if alias != locations_history_dates[0].location.alias:
                        # find the index of the matching location
                        idx = next(i for i, lhd in enumerate(locations_history_dates) if lhd.location.alias == alias)
                        if idx:
                            lhd = locations_history_dates.pop(idx)
                            locations_history_dates.insert(0, lhd)

            return locations_history_dates

        except SystemError as err:
            return HistoryGraph._error('There was an error getting the locations history dates', err)

    def _get_locations_daily_history(self) -> List[PyDailyHistories] | None:
        """
        Get the daily histories for each of the locations.
        """
        try:
            elapsed = Stopwatch()
            locations_daily_histories = []
            date_range = self._graph_selection.date_range
            for location_history_dates in self._locations_history_dates:
                data_criteria = PyLocationFilter(name=location_history_dates.location.alias)
                locations_daily_histories.append(
                    self._weather_data.backend.get_daily_history(data_criteria, date_range)
                )

            # make sure the primary location daily histories is first
            if self._graph_selection.primary_location > 0 and len(locations_daily_histories) > 1:
                # the list of daily histories are sorted so remove the primary and insert it at the head
                primary_location = locations_daily_histories.pop(self._graph_selection.primary_location)
                locations_daily_histories.insert(0, primary_location)
            log.info('_get_locations_daily_history %s', elapsed)
            return locations_daily_histories
        except SystemError as err:
            return HistoryGraph._error('There was an error getting the locations daily histories', err)

    def _select_graph(self):
        """
        Create the popup context menu that selects which graph to create.
        """
        elapsed = Stopwatch()
        locations_daily_histories = self._get_locations_daily_history()

        # replace the graph
        def set_graph_view(graph):
            self._graph_view = GraphView(self._parent, graph)

        elapsed_graph = Stopwatch()
        graph_type = self._graph_selection.graph_type
        if graph_type == GraphType.TEMPERATURE:
            set_graph_view(self._graphs.temperatures(self._graph_selection.temperature_type, locations_daily_histories))
        elif graph_type == GraphType.PRECIPITATION:
            set_graph_view(self._graphs.precipitation(self._graph_selection.precipitation_type,
                                                      locations_daily_histories))
        elif graph_type == GraphType.CONDITIONS:
            set_graph_view(self._graphs.conditions(self._graph_selection.conditions_type, locations_daily_histories))
        else:
            print('Yikes... There is no graph view to change!')
            return
        log.info('elapsed graph %s', elapsed_graph)

        def popup(event):
            try:
                menu.tk_popup(event.x_root, event.y_root)
            finally:
                menu.grab_release()

        menu = tk.Menu(self._graph_view, tearoff=0)

        # call the graph selector before changing the view
        def select_graph():
            GraphSelector(self._parent, self._graph_selection, self._locations_history_dates)
            if not self._graph_selection:
                return
            self._select_graph()

        menu.add_command(label="Change Graph", command=select_graph)
        self._graph_view.add_handler("<Button-3>", popup)
        if len(self._locations_history_dates) > 1:
            name = 'Locations History Graphs'
        else:
            name = f'{self._locations_history_dates[0].location.name} History Graphs'
        self._add_tab(name, self)

        log.info('Change View %s', elapsed)
