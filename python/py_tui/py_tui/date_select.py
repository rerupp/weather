import logging as log
from dataclasses import dataclass
from datetime import date, datetime
from enum import IntFlag

from py_weather_lib import PyDateRange
from textual import on, work
from textual.app import App, ComposeResult
from textual.containers import Center, CenterMiddle, Container, Horizontal, Vertical
from textual.messages import Message
from textual.reactive import reactive
from textual.screen import ModalScreen
from textual.widgets import Button, Footer, Header, Input, Label, MaskedInput


class DateInput(Horizontal):
    """ A date input widget.
    The date format is currently hardcoded as MM-DD-YYYY. Messages are sent
    when the date is changed and when the date is not valid.
    """

    @dataclass
    class Invalid(Message):
        """This message is sent when the date is not valid."""
        input: 'DateInput'

    @dataclass()
    class Changed(Message):
        """This message is sent when the date is valid and changed."""
        input: 'DateInput'

    DEFAULT_CSS = """
    DateInput {
        #label {
            color: $text-secondary;
        }
        #value {
            width: 11;
            height: 1;
            align: center middle;
            margin-left: 1;
        }
    }
    """
    DATE_FMT = "%m-%d-%Y"
    PLACEHOLDER = "MM-DD-YYYY"
    VALUE_ID = "#value"

    """The current date if the contents of the input widget is valid otherwise None. The value is reactive."""
    value: reactive[date | None] = reactive(None)

    def __init__(self, label: str, value: date | None = None, id: str | None = None, classes: str | None = None):
        super().__init__(id=id, classes=classes)
        self.label = label if label[-1] == ':' else label + ":"
        self.set_reactive(DateInput.value, value)

    def __repr__(self):
        return f"DateInput({self.label!r}, {self.value!r})"

    def compose(self) -> ComposeResult:
        yield Label(self.label, id="label")
        yield MaskedInput(
            id=self.VALUE_ID[1:],
            # leave the cursor at cursor_position when focused
            select_on_focus=False,
            value=date.strftime(self.value, self.DATE_FMT) if self.value else None,
            template="99-99-9999",
            compact=True,
            placeholder=self.PLACEHOLDER,
        )

    def watch_value(self) -> None:
        """When the current date changes update the input widget contents."""
        masked_input = self.query_one(self.VALUE_ID, MaskedInput)
        if not self.value:
            masked_input.clear()
        else:
            masked_input.replace(date.strftime(self.value, self.DATE_FMT), 0, len(self.PLACEHOLDER))
            masked_input.cursor_position = 0

    def on_mount(self) -> None:
        # if there is an initial value force the cursor to home
        if self.value:
            self.query_one(MaskedInput).cursor_position = 0

    @on(Input.Changed)
    def _input_changed(self, event: Input.Changed) -> None:
        """Validate the input widget date is valid and post the appropriate message. The Input.Changed event is stopped."""
        event.stop()
        # check that content matches the template before parsing the date
        if event.validation_result.is_valid:
            try:
                value = datetime.strptime(event.value, self.DATE_FMT).date()
                if value != self.value:
                    self.set_reactive(DateInput.value, value)
                    self.post_message(DateInput.Changed(self))
            except ValueError:
                self.post_message(DateInput.Invalid(self))


class DateRangeInput(Vertical):
    """A DateRange input widget.
    This component manages a start DateInput and end DateInput widget. A message is sent when the
    dates change with the state of the two dates.
    """

    class State(IntFlag):
        """The states associated with the input."""

        """The start and end date are valid."""
        OK = 0

        """The start date is not available."""
        START_UNDF = 1
        """The start date is not valid."""
        START_INVALID = 2
        """The start date is prior to the minimum date allowed."""
        START_PRIOR = 4
        """The start date is past the maximum date allowed."""
        START_PAST = 8

        """The end date is not available."""
        END_UNDF = 64
        """The end date is not valid."""
        END_INVALID = 128
        """The end date is prior to the minimum date allowed."""
        END_PRIOR = 256
        """The end date is past the maximum date allowed."""
        END_PAST = 512

        """The start date is past the end date."""
        START_PAST_END = 4096

        """The start date flag group."""
        START_FLAGS = START_UNDF | START_INVALID | START_PRIOR | START_PAST
        """The end date flag group."""
        END_FLAGS = END_UNDF | END_INVALID | END_PRIOR | END_PAST

    @dataclass()
    class Changed(Message):
        """The message send when the date range changes."""
        dates_input: 'DateRangeInput'
        state: 'DateRangeInput.State'

    DEFAULT_CSS = """
    DateRangeInput {
        #start {
            width: auto;
            height: auto;
        }
        #end {
            width: auto;
            height: auto;
            offset: 1 0;
        }
    }
    """
    START_ID = '#start'
    END_ID = '#end'

    def __init__(
            self,
            id: str | None = None,
            classes: str | None = None,
            start: date | None = None,
            end: date | None = None,
            min_date: date = date.min,
            max_date: date = date.max,
    ):
        super().__init__(id=id, classes=classes)
        self._start = start
        self._end = end
        self._min_date = min_date
        self._max_date = max_date

        # remember the current state
        self._state = DateRangeInput.State.OK
        if not start:
            self._state |= DateRangeInput.State.START_UNDF
        if not end:
            self._state |= DateRangeInput.State.START_UNDF
        self._state = self._get_state()

    def compose(self) -> ComposeResult:
        yield Center(DateInput("Start:", value=self._start, id=self.START_ID[1:]))
        yield Center(DateInput("End:", value=self._end, id=self.END_ID[1:]))

    @property
    def value(self) -> PyDateRange | None:
        return PyDateRange(start=self._start, end=self._end) if self._state == self.State.OK else None

    @value.setter
    def value(self, date_range: PyDateRange | None) -> None:
        # initialize the new state
        if date_range:
            self._start = date_range.start
            self._end = date_range.end
            self._state = self.State.OK
            self._state = self._get_state()
        else:
            self._start = self._end = None
            self._state |= self.State.START_UNDF | self.State.END_UNDF

        # initialize date inputs
        self.query_one(self.START_ID, DateInput).value = self._start
        self.query_one(self.END_ID, DateInput).value = self._end

        # now tell whoever is interested the dates changed regardless of the previous state
        self.post_message(self.Changed(self, self._state))

    @on(DateInput.Changed)
    def _date_changed(self, event: DateInput.Changed) -> None:
        # whoami = f"{self.__class__.__name__}.{inspect.currentframe().f_code.co_name}"
        event.stop()
        if event.input.id == self.START_ID[1:]:
            # if valid and at the end focus the end date
            if event.input.query_one(MaskedInput).cursor_at_end:
                end = self.query_one(self.END_ID, DateInput).query_one(MaskedInput)
                end.cursor_position = 0
                end.focus()
            if self._start != event.input.value:
                self._start = event.input.value
                # log.debug(f"{whoami} {self._start} ")
        else:
            if self._end != event.input.value:
                self._end = event.input.value
                # log.debug(f"{whoami} {self._end} ")

        current_state = self._get_state()
        if current_state != self._state or current_state == self.State.OK:
            self._state = current_state
            self.post_message(self.Changed(self, current_state))

    @on(DateInput.Invalid)
    def _date_invalid(self, event: DateInput.Invalid) -> None:
        event.stop()
        state = self._get_state()
        log.debug(f"{self.__class__.__name__}._date_invalid: {event} {state}")
        if event.input.id == self.START_ID[1:]:
            state &= self.State.END_FLAGS
            state |= self.State.START_INVALID
        else:
            state &= self.State.START_FLAGS
            state |= self.State.END_INVALID

        log.debug(f"{self.__class__.__name__}._date_invalid: current={state} previous{self._state}")
        if state != self._state:
            self._state = state
            self.post_message(self.Changed(self, state))

    def _get_state(self) -> 'DateRangeInput.State':
        state = DateRangeInput.State.OK

        start = self._start
        end = self._end
        if start is None:
            # restore the start date being invalid
            if self._state & self.State.START_INVALID:
                state |= self.State.START_INVALID
            else:
                state |= self.State.START_UNDF
        elif end and start > end:
            state |= self.State.START_PAST_END
        elif start < self._min_date:
            state |= self.State.START_PRIOR
        elif start > self._max_date:
            state |= self.State.START_PAST

        if end is None:
            # restore the end date being invalid
            if self._state & self.State.END_INVALID:
                state |= self.State.END_INVALID
            else:
                state |= self.State.END_UNDF
        elif start and end < start:
            state |= self.State.START_PAST_END
        elif end < self._min_date:
            state |= self.State.END_PRIOR
        elif end > self._max_date:
            state |= self.State.END_PAST

        return state


class DateRangeSelect(Vertical):
    @dataclass
    class Changed(Message):
        date_range: PyDateRange

    @dataclass
    class Invalid(Message):
        description: str

    DEFAULT_CSS = """
    DateRangeSelect {
        width: auto;
        height: auto;
        #banner {
            color: $text-primary;
        }
        #date-range-input {
            width: auto;
            height: 2;
        }
    }
    """
    DATERANGE_INPUT_ID = "#date-range-input"

    def __init__(
            self,
            id: str | None = None,
            classes: str | None = None,
            start_date: date | None = None,
            end_date: date | None = None,
            min_date: date = date(1970, 1, 1),
            max_date: date = date.today(),
    ) -> None:
        super().__init__(id=id, classes=classes)
        self._start_date = start_date
        self._end_date = end_date
        self._min_date = min_date
        self._max_date = max_date

    def compose(self) -> ComposeResult:
        yield DateRangeInput(
            id=self.DATERANGE_INPUT_ID[1:],
            start=self._start_date,
            end=self._end_date,
            min_date=self._min_date,
            max_date=self._max_date,
        )

    def initial_focus(self):
        date_input = self.query_one(f"{self.DATERANGE_INPUT_ID} {DateRangeInput.START_ID} {DateInput.VALUE_ID}")
        date_input.focus()

    @on(DateRangeInput.Changed)
    def _date_range_changed(self, event: DateRangeInput.Changed) -> None:
        event.stop()
        state = event.state
        if state == DateRangeInput.State.OK:
            start_date = self.query_one(DateRangeInput.START_ID, DateInput).value
            end_date = self.query_one(DateRangeInput.END_ID, DateInput).value
            if self._start_date != start_date or self._end_date != end_date:
                self._start_date = start_date
                self._end_date = end_date
                self.post_message(self.Changed(PyDateRange(start_date, end_date)))
        else:
            if state == DateRangeInput.State.START_INVALID + DateRangeInput.State.END_INVALID:
                description = 'Start/end dates not valid.'
            elif state & DateRangeInput.State.START_INVALID:
                description = 'The start date is not valid.'
            elif state & DateRangeInput.State.END_INVALID:
                description = 'The end date is not valid.'

            elif state == DateRangeInput.State.START_PRIOR + DateRangeInput.State.END_PRIOR:
                description = "Start and end dates are too early."
            elif state & DateRangeInput.State.START_PRIOR:
                description = "The start date is too early."
            elif state & DateRangeInput.State.END_PRIOR:
                description = "The end date is too early."

            elif state == DateRangeInput.State.START_PAST + DateRangeInput.State.END_PAST:
                description = "Start and end dates are too late."
            elif state & DateRangeInput.State.START_PAST:
                description = "The start date is too late."
            elif state & DateRangeInput.State.END_PAST:
                description = "The end date is too late."

            elif state & DateRangeInput.State.START_PAST_END:
                description = 'Start date after end date.'

            elif state | DateRangeInput.State.START_UNDF + DateRangeInput.State.END_UNDF:
                # at this point if one of the dates are not complete this clears previous errors
                description = ''
            else:
                description = f'Unknown error {state:04x}.'

            self.post_message(DateRangeSelect.Invalid(description))

    def start_date(self) -> date | None:
        return self._start_date

    def end_date(self) -> date | None:
        return self._end_date

    def date_range(self) -> PyDateRange | None:
        return PyDateRange(self._start_date, self._end_date) if self._start_date and self._end_date else None


class DateSelectScreen(ModalScreen):
    DEFAULT_CSS = """
        DateSelectScreen {
            align: center middle;
            #dialog {
                width: 40;
                height: auto;
                padding: 1 1 0 1;
                border: panel $text-primary;
                border-title-align: center;
            }
            #date-range-select {
                width: auto;
                height: auto;
                padding: 1;
                border: solid white 30%;
                border-title-color: $text-secondary;
                border-title-style: bold;
                border-title-align: center;
                border-subtitle-color: $error;
                border-subtitle-align: center;
            }
            #buttons {
                width: auto;
                height: 1;
                margin-top: 1;
                #ok {
                    margin-right: 1;
                }
                #cancel {
                    margin-left: 1;
                }
                Button {
                    width: 10;
                    height: 1;
                }
            }
        }
    """
    DATE_SELECT_ID = "#date-range-select"

    def __init__(self, title="Date Selection", dates_banner="Date Range", start_date: date | None = None,
                 end_date: date | None = None) -> None:
        super().__init__()
        self._title = title
        self._dates_banner = dates_banner
        self._start_date = start_date
        self._end_date = end_date

    def compose(self) -> ComposeResult:
        dates = DateRangeSelect(id=self.DATE_SELECT_ID[1:], start_date=self._start_date, end_date=self._end_date)
        dates.border_title = self._dates_banner
        buttons = Center(
            Horizontal(
                Button("Ok", variant="primary", id="ok", compact=True, disabled=True),
                Button("Cancel", variant="warning", id="cancel", compact=True),
                id="buttons"
            )
        )

        dialog = Container(dates, buttons, id="dialog")
        dialog.border_title = self._title
        yield CenterMiddle(dialog)

    @on(Button.Pressed)
    def on_button_pressed(self, event: Button.Pressed) -> PyDateRange | None:
        event.stop()
        log.debug("on_button_pressed")
        if event.button.id == "ok":
            self.dismiss(self.query_one(self.DATE_SELECT_ID, DateRangeSelect).date_range())
        else:
            self.dismiss(None)

    @on(DateRangeSelect.Changed)
    def _date_range_changed(self, event: DateRangeSelect.Changed) -> None:
        event.stop()
        self._start_date = event.date_range.start
        self._end_date = event.date_range.end
        self.query_one("#ok", Button).disabled = False
        self.query_one(self.DATE_SELECT_ID).border_subtitle = ""

    @on(DateRangeSelect.Invalid)
    def _date_range_invalid(self, event: DateRangeSelect.Invalid) -> None:
        event.stop()
        self.query_one("#ok", Button).disabled = True
        self.query_one(self.DATE_SELECT_ID).border_subtitle = event.description


if __name__ == "__main__":
    class TestBed(App):
        ENABLE_COMMAND_PALETTE = False
        DEFAULT_CSS = """
        Screen {
            width: auto;
            height: auto;
        }
        """

        def compose(self) -> ComposeResult:
            yield Header()
            yield Footer()
            with CenterMiddle():
                yield Center(Button("Show Dialog"))
                yield Center(Label("Select results", id="results"))

        @on(Button.Pressed)
        def _show_dialog(self, event: Button.Pressed):
            self.action_show_dialog()

        @work
        async def action_show_dialog(self) -> None:
            log.debug("show_dialog")
            result = await self.push_screen_wait(DateSelectScreen())
            date_range = "Canceled" if result is None else f"{result}"
            self.query_one(Label).update(date_range, layout=True)


    log.basicConfig(
        filename='testbed.log',
        filemode='w',
        format='%(asctime)s %(module)s[%(lineno)d]: %(message)s',
        datefmt='%H:%M:%S',
        level=log.DEBUG,
    )
    TestBed().run()
