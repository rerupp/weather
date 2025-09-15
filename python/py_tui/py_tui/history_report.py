import inspect
import logging as log
from datetime import datetime
from enum import IntFlag

import pytz
from py_weather_lib import (PyDailyHistories, PyDateRange, PyHistory, PyLocationFilter, PyWeatherData)
from pytz import timezone
from rich.text import Style, Text
from textual import events, on
from textual.app import App, ComposeResult
from textual.containers import Center, Container, Horizontal, Vertical
from textual.events import Mount
from textual.message import Message
from textual.reactive import var
from textual.widgets import (Button, Collapsible, DataTable, Footer, Header, SelectionList)
from textual.widgets.selection_list import Selection

from .date_select import DateRangeSelect


class ReportCategories(IntFlag):
    NONE = 0,
    TEMPERATURE = 1,
    PRECIPITATION = 2,
    WIND = 4,
    CONDITIONS = 8,
    CELESTIAL = 16,
    SUMMARY = 32,


class ReportSelection(Vertical):
    class Changed(Message):
        def __init__(self, date_range: PyDateRange, categories: ReportCategories):
            super().__init__()
            self.date_range = date_range
            self.categories = categories

        def __repr__(self):
            return f"ReportSelection({self.date_range}, {self.categories!r})"

    DEFAULT_CSS = """
    ReportSelection {
        width: 40;
        height: auto;
        margin-right: 1;
        #date-selection {
            padding-top: 1;
            padding-bottom: 1;
            width: auto;
            height: auto;
            border: solid white 30%;
            border-title-color: $text-secondary;
            border-title-style: bold;
            border-title-align: center;
            border-subtitle-color: $error;
            border-subtitle-align: center;
        }
        #category-selection {
            width: 100%;
            height: auto;
            margin-top: 1;
            border: solid white 30%;
            border-title-color: $text-secondary;
            border-title-style: bold;
            border-title-align: center;
            border-subtitle-color: $error;
            border-subtitle-align: center;
        }
        #category-selection-list {
            margin-top: 1;
            margin-bottom: 1;
            width: auto;
            height: auto;
        }
        #generate-report {
            width: auto;
            height: 1;
            margin-top: 1;
        }
    }
    """
    DATE_SELECTION_ID = "#date-selection"

    def __init__(self, categories: ReportCategories, id: str | None = None, classes: str | None = None):
        self._selection_invalid = False
        self._dates_invalid = True
        self._categories = categories
        super().__init__(id=id, classes=classes)

    def compose(self) -> ComposeResult:
        date_selection = DateRangeSelect(id=self.DATE_SELECTION_ID[1:])
        date_selection.border_title = "Report Dates"
        yield date_selection

        categories = self._categories
        category_selection = Center(
            SelectionList[int](
                Selection("Temperature", 1, bool(categories & categories.TEMPERATURE)),
                Selection("Precipitation", 2, bool(categories & categories.PRECIPITATION)),
                Selection("Wind", 4, bool(categories & categories.WIND)),
                Selection("Conditions", 8, bool(categories & categories.CONDITIONS)),
                Selection("Celestial", 16, bool(categories & categories.CELESTIAL)),
                Selection("Summary", 32, bool(categories & categories.SUMMARY)),
                id="category-selection-list",
                compact=True,
            ),
            id="category-selection"
        )
        category_selection.border_title = "Report Categories"
        yield category_selection

        yield Center(
            Button("Generate Report", variant="primary", id="generate-report", compact=True, disabled=True)
        )

    def initial_focus(self):
        self.query_one(self.DATE_SELECTION_ID, DateRangeSelect).initial_focus()

    @on(SelectionList.SelectedChanged)
    def _selection_changed(self, event: SelectionList.SelectedChanged) -> None:
        category_selection = self.query_one("#category-selection", Center)
        if len(event.selection_list.selected) == 0:
            category_selection.border_subtitle = "One category must be selected."
            self._selection_invalid = True
        else:
            category_selection.border_subtitle = ""
            self._selection_invalid = False
        self.query_one("#generate-report", Button).disabled = self._selection_invalid or self._dates_invalid

    @on(DateRangeSelect.Changed)
    def _date_range_changed(self) -> None:
        self.query_one(self.DATE_SELECTION_ID, DateRangeSelect).border_subtitle = ""
        self._dates_invalid = False
        self.query_one("#generate-report", Button).disabled = self._selection_invalid

    @on(DateRangeSelect.Invalid)
    def _date_range_invalid(self, event: DateRangeSelect.Invalid) -> None:
        self.query_one(self.DATE_SELECTION_ID, DateRangeSelect).border_subtitle = event.description
        self.query_one("#generate-report", Button).disabled = True
        self._dates_invalid = True

    @on(Button.Pressed)
    def _button_pressed(self) -> None:
        whoami = f"{self.__class__.__name__}.{inspect.currentframe().f_code.co_name}"
        date_range = self.query_one(self.DATE_SELECTION_ID, DateRangeSelect).date_range()
        selection = self.query_one("#category-selection-list", SelectionList).selected
        log.debug(f"{whoami}: {date_range} {selection!r}")
        report_categories = ReportCategories(0)
        for category in selection:
            report_categories |= ReportCategories(category)
        message = self.Changed(date_range, report_categories)
        self.post_message(message)


class ReportTable(Container):
    daily_histories: var[PyDailyHistories | None] = var(None)
    categories: var[ReportCategories] = var(~ReportCategories(0))

    def __init__(self, categories: ReportCategories, id: str | None = None, classes: str | None = None):
        super().__init__(
            DataTable(id="report-contents", fixed_columns=1, header_height=2, cursor_type="row"),
            id=id,
            classes=classes
        )
        self.set_reactive(ReportTable.categories, categories)

    @on(Mount)
    def _on_mount(self, event: events.Mount) -> None:
        self._init_table()

    def watch_daily_histories(self):
        self._init_table()

    def watch_categories(self):
        self._init_table()

    def init_report(self, daily_histories: PyDailyHistories, categories: ReportCategories):
        self.set_reactive(ReportTable.daily_histories, daily_histories)
        self.set_reactive(ReportTable.categories, categories)
        self._init_table()

    def _init_table(self):
        label_style = self._label_style()
        table = self.query_one(DataTable)
        table.clear(columns=True)
        columns = self._get_columns(label_style)
        table.add_columns(*tuple(columns))
        if self.daily_histories is not None:
            tz = pytz.timezone(self.daily_histories.location.tz)
            for history in self.daily_histories.histories:
                row = self._get_row(history, label_style, tz)
                table.add_row(*row)

    def _get_columns(self, style: Style) -> list[Text]:
        columns = [
            Text("\n".join([
                "",
                " Date "
            ]), style=style)
        ]
        if self.categories & ReportCategories.TEMPERATURE:
            columns.append(
                Text("\n".join([
                    "--- Temperature ---",
                    " High   Low    Mean"
                ]), style=style)

            )
        if self.categories & ReportCategories.PRECIPITATION:
            columns.append(
                Text("\n".join([
                    "----- Precipitation ------",
                    "Chance Amount     Type"
                ]), style=style)

            )
        if self.categories & ReportCategories.WIND:
            columns.append(
                Text("\n".join([
                    "------- Wind -------",
                    "Speed   Gust Bearing"
                ]), style=style)
            )
        if self.categories & ReportCategories.CONDITIONS:
            columns.append(
                Text("\n".join([
                    " Dew  Cloud                      UV",
                    "Point Cover Humidity Pressure   Index"
                ]), style=style)
            )
        if self.categories & ReportCategories.CELESTIAL:
            columns.append(
                Text("\n".join([
                    "--------- Celestial ----------",
                    "Sunrise Sunset   Moon Phase"
                ]), style=style)
            )
        if self.categories & ReportCategories.SUMMARY:
            columns.append(
                Text("\n".join([
                    "",
                    "Summary"
                ]), style=style)
            )
        return columns

    def _get_row(self, history: PyHistory, label_style: Style, tz: timezone) -> list[Text]:
        row = [
            Text(history.date.strftime('%b-%d'), style=label_style)
        ]
        if self.categories & ReportCategories.TEMPERATURE:
            high = f"{history.temperature_high:.1f}" if history.temperature_high is not None else ""
            low = f"{history.temperature_low:.1f}" if history.temperature_low is not None else ""
            mean = f"{history.temperature_mean:.1f}" if history.temperature_mean is not None else ""
            row.append(Text(f"{high:>5}  {low:>5}  {mean:>5}"))
        if self.categories & ReportCategories.PRECIPITATION:
            chance = "" if history.precipitation_chance is None else f"{history.precipitation_chance:.0%}"
            amount = "" if history.precipitation_amount is None else f"{history.precipitation_amount:.1f}"
            desc = "" if history.precipitation_type is None else history.precipitation_type
            row.append(Text(f"{chance:>5}  {amount:>5}  {desc}"))
        if self.categories & ReportCategories.WIND:
            speed = "" if history.wind_speed is None else f"{history.wind_speed:.1f}"
            gust = "" if history.wind_gust is None else f"{history.wind_gust:.1f}"
            direction = self._wind_bearing(history.wind_direction)
            row.append(Text(f"{speed:>5}  {gust:>5}   {direction:^3}"))
        if self.categories & ReportCategories.CONDITIONS:
            dew_point = "" if history.dew_point is None else f"{history.dew_point:.1f}"
            cloud_cover = "" if history.cloud_cover is None else f"{history.cloud_cover:.0%}"
            humidity = "" if history.humidity is None else f"{history.humidity:.0%}"
            pressure = "" if history.pressure is None else f"{history.pressure:.1f}"
            uv_index = self._uv_index(history.uv_index)
            row.append(Text(f"{dew_point:>5}   {cloud_cover:>4}   {humidity:>4}    {pressure:>6}  {uv_index:^9}"))
        if self.categories & ReportCategories.CELESTIAL:
            sunrise = "" if history.sunrise is None else self._local_time(history.sunrise, tz)
            sunset = "" if history.sunset is None else self._local_time(history.sunset, tz)
            moon_phase = self._moon_phase(history.moon_phase)
            row.append(Text(f" {sunrise}   {sunset} {moon_phase:^15}"))
        if self.categories & ReportCategories.SUMMARY:
            row.append(Text("" if history.description is None else history.description.strip()))
        return row

    def _label_style(self) -> Style:
        theme_variables = self.app.theme_variables
        return Style(color=theme_variables["text-primary"])

    @staticmethod
    def _moon_phase(moon_phase: float | None = None) -> str:
        if 0.0 <= moon_phase <= 0.01:
            return 'new moon'
        if 0.01 < moon_phase < 0.24:
            return 'waxing crescent'
        if 0.24 <= moon_phase <= 0.26:
            return 'first quarter'
        if 0.26 < moon_phase < 0.49:
            return "waxing gibbous"
        if 0.49 <= moon_phase <= 0.51:
            return 'full moon'
        if 0.51 < moon_phase < 0.74:
            return 'waning gibbous'
        if 0.74 <= moon_phase <= 0.76:
            return 'last quarter'
        if 0.76 < moon_phase <= 1.0:
            return 'waning crescent'
        if moon_phase > 1.0:
            return 'unknown'
        return ''

    @staticmethod
    def _uv_index(uv_index: float | None = None) -> str:
        if uv_index is None:
            return ''
        match round(uv_index):
            case 1 | 2:
                return 'low'
            case 3 | 4 | 5:
                return 'moderate'
            case 6 | 7:
                return "high"
            case 8 | 9 | 10:
                return "very high"
            case _:
                return "extreme"

    @staticmethod
    def _wind_bearing(bearing: int | None = None) -> str:
        if bearing is None:
            return ""
        match int(round((float(bearing) / 22.5))) % 16:
            case 0:
                return "N"
            case 1:
                return "NNE"
            case 2:
                return "NE"
            case 3:
                return "ENE"
            case 4:
                return "E"
            case 5:
                return "ESE"
            case 6:
                return "SE"
            case 7:
                return "SSE"
            case 8:
                return "S"
            case 9:
                return "SSW"
            case 10:
                return "SW"
            case 11:
                return "WSW"
            case 12:
                return "W"
            case 13:
                return "WNW"
            case 14:
                return "NW"
            case _:
                return "NNW"

    @staticmethod
    def _local_time(utc: datetime, tz: timezone) -> str:
        local_dt = utc.replace(tzinfo=pytz.utc).astimezone(tz)
        return local_dt.strftime("%H:%M")


class HistoryReport(Horizontal):
    SELECTION_ID = "#selection"

    def __init__(self, alias: str, weather_data: PyWeatherData, id: str | None = None,
                 classes: str | None = None):
        self._alias = alias
        self._weather_data = weather_data
        self._date_range: PyDateRange | None = None
        self._categories = ~ReportCategories(0)
        super().__init__(id=id, classes=classes)

    def compose(self) -> ComposeResult:
        yield ReportSelection(self._categories, id=self.SELECTION_ID[1:])
        yield ReportTable(self._categories, id="table")

    def initial_focus(self):
        self.query_one(self.SELECTION_ID, ReportSelection).initial_focus()

    @on(ReportSelection.Changed)
    def _selection_changed(self, selection: ReportSelection.Changed):
        # log.debug(f"_selection_changed: {selection}")
        if self._date_range is None:
            dates_changed = True
            self._date_range = selection.date_range
        elif self._date_range.start != selection.date_range.start or self._date_range.end != selection.date_range.end:
            dates_changed = True
            self._date_range = selection.date_range
        else:
            dates_changed = False
        if self._categories != selection.categories:
            self._categories = selection.categories
            categories_changed = True
        else:
            categories_changed = False

        table = self.query_one("#table", ReportTable)
        if dates_changed and categories_changed:
            daily_histories = self._get_histories()
            table.init_report(daily_histories, self._categories)
        elif dates_changed:
            daily_histories = self._get_histories()
            table.daily_histories = daily_histories
        elif categories_changed:
            table.categories = self._categories

    def _get_histories(self) -> PyDailyHistories:
        return self._weather_data.get_daily_history(PyLocationFilter(name=self._alias), self._date_range)


if __name__ == "__main__":
    from py_weather_lib import create, PyWeatherConfig


    class LocationReportApp(App):
        ENABLE_COMMAND_PALETTE = False
        DEFAULT_CSS = """
        #location-report {
            width: 100%;
            height: 24;
        }
        """

        def __init__(self, weather_data: PyWeatherData):
            self._weather_data = weather_data
            super().__init__(watch_css=True)

        def compose(self) -> ComposeResult:
            yield Header()
            yield Footer()
            yield Collapsible(
                Collapsible(
                    HistoryReport("foothills", self._weather_data, id="location-report"),
                    collapsed=False
                ),
                collapsed=False
            )


    log.basicConfig(
        filename='testbed.log',
        filemode='w',
        format='%(asctime)s %(name)s %(lineno)d: %(message)s',
        datefmt='%H:%M:%S',
        level=log.DEBUG,
    )
    LocationReportApp(create(PyWeatherConfig(dirname="../../rust/weather_data"))).run()
