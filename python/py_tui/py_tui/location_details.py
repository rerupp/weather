import logging as log

from py_weather_lib import (PyDailyHistories, PyDateRange, PyHistoryDates, PyLocation, PyLocationFilter,
                            PyLocationFilters, PyWeatherData)
from textual import on
from textual.app import App, ComposeResult
from textual.containers import Center, CenterMiddle, Horizontal, Right, Vertical, VerticalGroup
from textual.events import Message, Mount
from textual.widgets import Button, Footer, Header, Input, Label, ListItem, ListView

from .message_box import MessageBox
from .new_histories import NewHistories


class LocationProperties(Vertical):
    DEFAULT_CSS = """
    LocationProperties {
        width: auto;
        height: auto;
        align: center middle;
        #properties-container {
            align: center middle;
            width: auto;
            height: auto;
        }
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
        #modify-properties {
            width: 100%;
            height: auto;
            padding-bottom: 1;
        }
    }
    """

    def __init__(self, id: str | None = None, classes: str | None = None, location: PyLocation | None = None):
        super().__init__(id=id, classes=classes)
        self._location = location

    def compose(self) -> ComposeResult:
        with Vertical(id="properties-container"):
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
            with Center(id="modify-properties"):
                yield Button("Modify", id="modify", compact=True, variant="primary")


class HistoryList(Vertical):
    class AddHistories(Message):
        def __init__(self, result: PyDailyHistories | str | None):
            super().__init__()
            self.result = result

    DEFAULT_CSS = """
    HistoryList {
        width: auto;
        height: auto;
        align: center middle;
        #history-container {
            align: center middle;
            width: auto;
            height: auto;
        }
        #history-view {
            width: 30;
            height: 5;
            margin: 1;
            background: blue 15%;
        }
        #add-histories {
            align: center middle;
            width: 100%;
            height: auto;
            padding-bottom: 1;
        }
    }
    """

    def __init__(self, alias: str, history_dates: list[PyDateRange], weather_data: PyWeatherData, id: str | None = None,
                 classes: str | None = None):
        super().__init__(id=id, classes=classes)
        self._alias = alias
        self._weather_data = weather_data
        self._history_dates = history_dates

    def compose(self) -> ComposeResult:
        with VerticalGroup(id="history-container"):
            yield ListView(id="history-view")
            with Center(id="add-histories"):
                yield Button(" Add Histories", id="add", compact=True, variant="primary")

    @on(Mount)
    def _on_mount(self) -> None:
        self._update_history_dates()

    @on(Button.Pressed)
    def _new_histories(self) -> None:
        self.app.push_screen(NewHistories(self._alias, self._weather_data), self._add_history_callback)

    @on(AddHistories)
    def _add_histories(self, add_histories: AddHistories) -> None:
        if isinstance(add_histories.result, PyDailyHistories):
            try:
                added = self._weather_data.add_histories(add_histories.result)
                filters = PyLocationFilters([PyLocationFilter(name=self._alias)])
                self._history_dates = self._weather_data.get_history_dates(filters)[0].history_dates
                self._update_history_dates()
                self.app.push_screen(MessageBox(f"{added} histories were added."))
            except SystemError as error:
                # set the error into result allowing it to be displayed
                add_histories.result = str(error)
        if isinstance(add_histories.result, str):
            self.app.push_screen(MessageBox(add_histories.result, MessageBox.Type.ERROR))

    def _update_history_dates(self) -> None:
        list_view = self.query_one(ListView)
        list_view.clear()
        for date_range in self._history_dates:
            list_view.append(ListItem(Label(str(date_range))))
        list_view.index = 0

    def _add_history_callback(self, result: PyDailyHistories | str | None) -> None:
        # the new histories screen calls this so post a message allowing self to capture it
        self.post_message(self.AddHistories(result))


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

    def __init__(self, location_history_dates: PyHistoryDates, weather_data: PyWeatherData, id: str | None = None,
                 classes: str | None = None):
        super().__init__(id=id, classes=classes)
        self._weather_data = weather_data
        self._location_history_dates = location_history_dates

    def compose(self) -> ComposeResult:
        properties = LocationProperties(id="location-properties", location=self._location_history_dates.location)
        properties.border_title = "Location Properties"
        yield properties
        histories = HistoryList(self._location_history_dates.location.alias, self._location_history_dates.history_dates,
                                self._weather_data, id="location-histories")
        histories.border_title = "History Dates"
        yield histories

    def initial_focus(self):
        buttons = [button for button in self.query(Button)]
        for button in buttons:
            if button.id == "modify":
                button.focus()


if __name__ == "__main__":
    from py_weather_lib import PyWeatherConfig, create


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

        def __init__(self, weather_data: PyWeatherData):
            super().__init__(watch_css=True)
            filters = PyLocationFilters([PyLocationFilter(name="foothills")])
            history_dates = weather_data.get_history_dates(filters)
            self._weather_data = weather_data
            self._history_dates = history_dates[0]

        def compose(self) -> ComposeResult:
            yield Header()
            yield Footer()
            with CenterMiddle():
                yield LocationDetails(self._history_dates, self._weather_data, id="location-details")


    log.basicConfig(
        filename='testbed.log',
        filemode='w',
        format='%(asctime)s: %(message)s',
        datefmt='%H:%M:%S',
        level=log.DEBUG,
    )

    LocationDetailsApp(create(PyWeatherConfig(dirname="../../rust/weather_data"))).run()
