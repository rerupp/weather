import logging as log

from py_weather_lib import PyHistoryDates, PyWeatherData
from textual import on
from textual.app import ComposeResult
from textual.widgets import Collapsible

from .history_report import HistoryReport
from .location_details import LocationDetails


class HistoryView(Collapsible):
    DEFAULT_CSS = """
    HistoryView {
        width: auto;
        height: auto;
        #report {
         width: 100%;
         height: 24;
        }
    }
    """

    LOCATION_DETAILS_ID = "#location-details"
    DETAILS_ID = "#details"
    LOCATION_REPORT_ID = "#location-report"
    REPORT_ID = "#report"

    def __init__(self, weather_data: PyWeatherData, history_dates: PyHistoryDates):
        super().__init__(id=f"{history_dates.location.alias}-view")
        self._weather_data = weather_data
        self._history_dates = history_dates

    def compose(self) -> ComposeResult:
        with Collapsible(id=self._history_dates.location.alias,
                        title=f"{self._history_dates.location.city_name} ({self._history_dates.location.region_code})",
                         collapsed=True):
            with Collapsible(id=self.LOCATION_DETAILS_ID[1:], title="Location Details", collapsed=False):
                yield LocationDetails(self._history_dates, self._weather_data, id=self.DETAILS_ID[1:])
            with Collapsible(id=self.LOCATION_REPORT_ID[1:], title="Location Report", collapsed=True):
                yield HistoryReport(self._history_dates.location.alias, self._weather_data, id=self.REPORT_ID[1:])

    @on(Collapsible.Expanded)
    def _on_expanded(self, event: Collapsible.Expanded):
        log.debug(f"expanded event: {event.control.id}")
        event.stop()
        if event.control.id == self.LOCATION_REPORT_ID[1:]:
            self.query_one(self.REPORT_ID, HistoryReport).initial_focus()
        elif event.control.id == self.LOCATION_DETAILS_ID[1:]:
            self.query_one(self.DETAILS_ID, LocationDetails).initial_focus()
        else:
            # if you get here the parent collapsible is expanded
            details = self.query_one(self.LOCATION_DETAILS_ID, Collapsible)
            if not details.collapsed:
                details.query_one(self.DETAILS_ID, LocationDetails).initial_focus()
                return
            report = self.query_one(self.LOCATION_REPORT_ID, Collapsible)
            if not report.collapsed:
                report.query_one(self.REPORT_ID, HistoryReport).initial_focus()
                return
            log.debug(f"both details and report are collapsed ({self})")
