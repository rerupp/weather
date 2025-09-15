import logging as log
from datetime import date

from py_weather_lib import PyHistoryDates, PyLocation, PyDateRange
from textual import on
from textual.app import App, ComposeResult
from textual.containers import Center, Horizontal, Right, Vertical, CenterMiddle
from textual.events import Mount
from textual.widgets import Footer, Header, Input, Label, ListItem, ListView


class LocationProperties(Vertical):
    DEFAULT_CSS = """
    LocationProperties {
        width: auto;
        height: auto;

        #properties {
            margin: 1;
            width: auto;
            height: auto;
            align: center middle;
        }
        .property {
            height: 1;
            width: auto;
        }
        Right {
            margin-left: 1;
            width: 10;
            height: 1;
        }
        .lhs {
            color: $text-secondary;
        }
        .rhs {
            margin-left: 1;
            margin-right: 1;
            width: auto;
            height: 1;
        }
    }
    """

    def __init__(self, id: str | None = None, classes: str | None = None, location: PyLocation | None = None):
        super().__init__(id=id, classes=classes)
        self._location = location

    def compose(self) -> ComposeResult:
        with Center(id="properties"):
            with Horizontal(classes="property"):
                yield Right(Label("Alias:", classes="lhs"))
                widget = Input(self._location.alias if self._location else "", id="alias", classes="rhs",
                               compact=True, placeholder="Alias name")
                widget.can_focus = False
                yield widget
            with Horizontal(classes="property"):
                yield Right(Label("State:", classes="lhs"))
                widget = Input(self._location.state if self._location else "", id="state", classes="rhs",
                               compact=True, placeholder="State name")
                widget.can_focus = False
                yield widget
            with Horizontal(classes="property"):
                yield Right(Label("Latitude:", classes="lhs"))
                widget = Input(self._location.latitude if self._location else "", id="lat", classes="rhs",
                               compact=True, placeholder=" ##.#########")
                widget.can_focus = False
                yield widget
            with Horizontal(classes="property"):
                yield Right(Label("Longitude:", classes="lhs"))
                widget = Input(self._location.longitude if self._location else "", id="long", classes="rhs",
                               compact=True, placeholder=" ###.#########")
                widget.can_focus = False
                yield widget
            with Horizontal(classes="property"):
                yield Right(Label("Timezone:", classes="lhs"))
                widget = Input(self._location.tz if self._location else "", id="tz", classes="rhs", compact=True,
                               placeholder="Timezone name")
                widget.can_focus = False
                yield widget


class HistoryList(Center):
    DEFAULT_CSS = """
    HistoryList {
        width: auto;
        height: auto;
        align: center middle;

        #list_view {
            width: 28;
            height: 5;
            margin: 1;
        }
    }
    """

    def __init__(self, history_dates: list[PyDateRange], id: str | None = None, classes: str | None = None):
        super().__init__(id=id, classes=classes)
        self._history_dates = history_dates

    def compose(self) -> ComposeResult:
        yield ListView(id="list_view")

    @on(Mount)
    def _on_mount(self) -> None:
        def fmt(d: date) -> str:
            return date.strftime(d, '%m-%d-%Y')

        list_view = self.query_one(ListView)
        for date_range in self._history_dates:
            list_view.append(ListItem(Label(f"{fmt(date_range.start)} thru {fmt(date_range.end)}")))
        list_view.index = 0


class LocationDetails(Horizontal):
    DEFAULT_CSS = """
    LocationDetails {
        width: auto;
        height: auto;
        #location-properties {
            width: auto;
            height: auto;
            border: solid white 50%;
            border-title-color: $text-secondary;
            border-title-style: bold;
            border-title-align: center;
        }
        #location-histories {
            width: auto;
            height: auto;
            margin-left: 1;
            border: solid white 50%;
            border-title-color: $text-secondary;
            border-title-style: bold;
            border-title-align: center;
        }
    }
    """
    CSS_PATH = "app.tcss"

    def __init__(self, location_history_dates: PyHistoryDates, id: str | None = None, classes: str | None = None):
        super().__init__(id=id, classes=classes)
        self._location_history_dates = location_history_dates

    def compose(self) -> ComposeResult:
        properties = LocationProperties(id="location-properties",
                                        location=self._location_history_dates.location)
        properties.border_title = "Location Properties"
        yield properties
        histories = HistoryList(self._location_history_dates.history_dates, id="location-histories")
        histories.border_title = "History Date Ranges"
        yield histories


if __name__ == "__main__":
    from py_weather_lib import PyWeatherConfig, PyWeatherData, PyLocationFilters, PyLocationFilter, create
    class LocationDetailsApp(App):
        ENABLE_COMMAND_PALETTE = False
        DEFAULT_CSS = """
        Screen {
            width: auto;
            height: auto;

            #location-details {
                width: auto;
                height: auto;
            }
        }
        """

        def __init__(self, history_dates: PyHistoryDates | None = None):
            super().__init__(watch_css=True)
            self._lhd = history_dates

        def compose(self) -> ComposeResult:
            yield Header()
            yield Footer()
            with CenterMiddle():
                yield LocationDetails(self._lhd, id="location-details")


    log.basicConfig(
        filename='testbed.log',
        filemode='w',
        format='%(asctime)s: %(message)s',
        datefmt='%H:%M:%S',
        level=log.DEBUG,
    )

    weather_data: PyWeatherData = create(PyWeatherConfig(dirname="../../rust/weather_data"))
    filters = PyLocationFilters([PyLocationFilter(name="foothills")])
    LocationDetailsApp(weather_data.get_history_dates(filters)[0]).run()
