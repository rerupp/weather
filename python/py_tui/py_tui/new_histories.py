import time

from py_weather_lib import PyDailyHistories, PyHistoriesFuture, PyLocationFilter, PyWeatherData
from textual import on, work
from textual.app import ComposeResult
from textual.containers import Center, Horizontal, VerticalGroup
from textual.message import Message
from textual.screen import ModalScreen
from textual.widgets import Button, Header, ProgressBar

from .date_select import DateRangeSelect


class NewHistories(ModalScreen[PyDailyHistories | str | None]):
    class Update(Message):
        def __init__(self, progress: float) -> None:
            super().__init__()
            self.progress = progress

    class Completed(Message):
        def __init__(self, result: PyDailyHistories | str) -> None:
            super().__init__()
            self.result = result

    CSS_PATH = "new_histories.tcss"

    DATE_SELECTION_ID = "#date-selection"
    PROGRESS_ID = "#progress"
    ADD_ID = "#add"
    CANCEL_ID = "#cancel"

    def __init__(self, alias: str, weather_data: PyWeatherData) -> None:
        super().__init__()
        self.title = "Add History"
        self._alias = alias
        self._weather_data = weather_data
        self._progress_total = 1000.0

    def compose(self) -> ComposeResult:
        yield Header()
        with VerticalGroup(id="dialog"):
            date_selection = DateRangeSelect(id=self.DATE_SELECTION_ID[1:])
            date_selection.border_title = "New History Dates"
            yield date_selection
            with Center():
                yield ProgressBar(total=self._progress_total, show_percentage=False, show_eta=False,
                                  id=self.PROGRESS_ID[1:])
            with Center():
                with Horizontal(id="buttons"):
                    yield Button("Add", compact=True, variant="primary", id=self.ADD_ID[1:], disabled=True)
                    yield Button("Cancel", compact=True, variant="error", id=self.CANCEL_ID[1:])

    @on(DateRangeSelect.Changed)
    def _date_range_changed(self) -> None:
        self.query_one(self.DATE_SELECTION_ID, DateRangeSelect).border_subtitle = ""
        add = self.query_one(self.ADD_ID, Button)
        add.disabled = False
        add.focus()

    @on(DateRangeSelect.Invalid)
    def _date_range_invalid(self, event: DateRangeSelect.Invalid) -> None:
        self.query_one(self.DATE_SELECTION_ID, DateRangeSelect).border_subtitle = event.description
        self.query_one(self.ADD_ID, Button).disabled = True

    @on(Update)
    def update_progress(self, update: Update) -> None:
        self.query_one(ProgressBar).update(progress=update.progress)

    @on(Completed)
    def _completed(self, completed: Completed) -> None:
        # when the screen is dismissed the callers callback will be invoked
        self.dismiss(completed.result)

    @on(Button.Pressed)
    def button_pressed(self, event: Button.Pressed):
        if event.button.id == self.CANCEL_ID[1:]:
            self.dismiss(None)
        else:
            self.query_one(self.ADD_ID, Button).disabled = True
            try:
                date_range = self.query_one(self.DATE_SELECTION_ID, DateRangeSelect).date_range()
                future = self._weather_data.new_daily_histories(PyLocationFilter(name=self._alias), date_range)
                self.wait_on_future(future)
            except SystemError as error:
                self.dismiss(str(error))

    @work(exclusive=True, thread=True)
    def wait_on_future(self, future: PyHistoriesFuture) -> None:
        sleep_interval = 0.001
        progress = 0.0
        advance = 1.0
        while not future.is_finished():
            progress += advance
            if progress >= self._progress_total or progress <= 0:
                advance *= -1
            if progress % 100.0 == 0.0:
                self.post_message(self.Update(progress))
            time.sleep(sleep_interval)
        # you need to post a message so the screen can be dismissed by the main thread
        try:
            self.post_message(self.Completed(future.get()))
        except SystemError as error:
            self.post_message(self.Completed(str(error)))
