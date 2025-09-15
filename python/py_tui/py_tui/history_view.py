from py_weather_lib import PyHistoryDates, PyWeatherData
from textual import on
from textual.app import ComposeResult
from textual.containers import Vertical
from textual.widgets import Collapsible

from .history_report import HistoryReport
from .location_details import LocationDetails


class HistoryView(Vertical):
    DEFAULT_CSS = """
    HistoryView {
        width: auto;
        height: auto;
        #report { width: 100%; height: 24; }
    }
    """

    def __init__(self, weather_data: PyWeatherData, history_dates: PyHistoryDates, id: str | None = None,
                 classes: str | None = None):
        super().__init__(id=id, classes=classes)
        self._weather_data = weather_data
        self._history_dates = history_dates

    def compose(self) -> ComposeResult:
        yield Collapsible(
            Collapsible(
                LocationDetails(self._history_dates, id="details"),
                title="Location Details", collapsed=False, id="collapsible-details"),
            Collapsible(
                HistoryReport(self._history_dates.location.alias, self._weather_data, id="report"),
                title="Location Report", collapsed=True, id="collapsible-report"),
            collapsed=True,
            title=self._history_dates.location.name,
        )

    @on(Collapsible.Expanded)
    def _on_expanded(self, event: Collapsible.Expanded):
        event.stop()
        if event.control.id == "collapsible-report":
            self.query_one("#report", HistoryReport).initial_focus()
