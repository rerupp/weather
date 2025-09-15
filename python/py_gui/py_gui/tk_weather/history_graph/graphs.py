from dataclasses import dataclass
from datetime import date
from enum import IntEnum, IntFlag
from typing import List

import matplotlib.dates as mdates
import matplotlib.pyplot as plt
from py_weather_lib import PyDailyHistories

__all__ = ['ConditionsType', 'FontSize', 'Graphs', 'PrecipitationType', 'TemperaturesType']


class PrecipitationType(IntEnum):
    """The type of precipitation graph to create."""
    UNKNOWN = 0,
    RAIN = 1,
    HUMIDITY = 2,
    CLOUD_COVER = 3,


class TemperaturesType(IntFlag):
    """The contents of the temperature graph."""
    UNKNOWN = 0,
    HIGH = 1,
    LOW = 2,
    MEAN = 4,


class ConditionsType(IntFlag):
    """The type of condition graph to create."""
    UNKNOWN = 0,
    WIND_SPEED = 1,
    WIND_GUST = 2,
    UV_INDEX = 4,


@dataclass(frozen=True)
class FontSize:
    """The font sizes used by figures."""
    title = 'small'
    legend = 'small'
    x_label = 'small'
    y_label = 'small'
    x_tick = 'x-small'
    y_tick = 'x-small'


class Graphs:
    def __init__(self, font_size=FontSize()):
        self._font_size = font_size

    def temperatures(self, temperature_type: TemperaturesType,
                     locations_daily_histories: List[PyDailyHistories]) -> plt.Figure:
        """Creates the temperatures history graph."""
        return TemperatureGraph(temperature_type, locations_daily_histories, self._font_size).figure

    def precipitation(self, precipitation_type: PrecipitationType,
                      locations_daily_histories: List[PyDailyHistories]) -> plt.Figure:
        """Creates the precipitations graph."""
        return PrecipitationGraph(precipitation_type, locations_daily_histories, self._font_size).figure

    def conditions(self, condition_type: ConditionsType,
                   locations_daily_histories: List[PyDailyHistories]) -> plt.Figure:
        """Creates the conditions graph."""
        return ConditionsGraph(condition_type, locations_daily_histories, self._font_size).figure


def _get_plot_dates(daily_histories: PyDailyHistories) -> List[date]:
    return [history.date for history in daily_histories.histories]


class BaseGraph:
    """This is the base class for the other graphs."""

    def __init__(self, title: str, ylabel: str, font_size: FontSize, include_legend=False):
        self._font_size = font_size
        self.figure = plt.Figure()
        self._ax = self.figure.add_subplot()

        # turn on the minor tick marks
        self._ax.minorticks_on()

        # set up the x-axis
        self._ax.grid(axis='x', which='major', linestyle='-', linewidth='0.5', color='gray')
        self._ax.grid(axis='x', which='minor', linestyle='dotted', linewidth='0.5', color='gray')

        # set up the y-axis
        self._ax.grid(axis='y', which='major', linestyle='-', linewidth='0.5', color='gray')
        self._ax.grid(axis='y', which='minor', linestyle='dotted', linewidth='0.5', color='gray')

        # configure the decorations
        self._ax.set_title(title, fontsize=self._font_size.title)
        self._ax.set_xlabel("Dates", fontsize=self._font_size.x_label)
        self._ax.set_ylabel(ylabel, fontsize=self._font_size.y_label)

        # set a minor grid line at the middle of the month
        self._ax.xaxis.set_minor_locator(mdates.DayLocator(interval=1, bymonthday=15))

        # set major grid lines for each month
        self._ax.xaxis.set_major_locator(mdates.MonthLocator())
        self._ax.xaxis.set_major_formatter(mdates.DateFormatter('%b %Y'))

        # set the size of the tick decorations
        self._ax.tick_params(axis='x', which='major', labelsize=self._font_size.x_tick)
        self._ax.tick_params(axis='y', which='major', labelsize=self._font_size.y_tick)

        self.add_plots()

        if include_legend:
            self.figure.legend(fontsize=self._font_size.legend)
        self.figure.autofmt_xdate()

    def add_plots(self):
        """This will be called from init to add in the plots."""
        pass


class TemperatureGraph(BaseGraph):
    def __init__(self, temperature_type: TemperaturesType, locations_daily_histories: List[PyDailyHistories],
                 font_size: FontSize):
        """Creates the temperatures history graph."""

        self._temperature_type = temperature_type
        self._locations_daily_histories = locations_daily_histories

        # create the graph title
        plot_types = []
        if temperature_type & TemperaturesType.HIGH:
            plot_types.append('High')
        if temperature_type & TemperaturesType.LOW:
            plot_types.append('Low')
        if temperature_type & TemperaturesType.MEAN:
            plot_types.append('Mean')
        plot_count = len(plot_types)
        if plot_count == 1:
            title = plot_types[0]
        elif plot_count == 2:
            title = ' and '.join(plot_types)
        elif plot_count > 2:
            title = ", ".join(plot_types[:-1]) + ' and ' + plot_types[-1]
        else:
            title = 'Unknown'

        # set up the label makers
        multiple_locations = len(locations_daily_histories) > 1
        multiple_plots = len(plot_types) > 1
        if not multiple_locations:
            self.__label = lambda _, w: w
        else:
            self.__label = lambda n, w: f'{n} {w}' if multiple_plots else n

        # create the plots
        BaseGraph.__init__(self, f'{title} Temperatures', 'Temperature (F)', font_size,
                           include_legend=multiple_locations or multiple_plots)

    def add_plots(self):
        """Called by the base class to create the temperature plots."""
        for daily_histories in self._locations_daily_histories:
            x = _get_plot_dates(daily_histories)
            if self._temperature_type & TemperaturesType.HIGH:
                y = [h.temperature_high for h in daily_histories.histories]
                self._ax.plot(x, y, label=self.__label(daily_histories.location.name, 'High'))
            if self._temperature_type & TemperaturesType.LOW:
                y = [h.temperature_low for h in daily_histories.histories]
                self._ax.plot(x, y, label=self.__label(daily_histories.location.name, 'Low'))
            if self._temperature_type & TemperaturesType.MEAN:
                y = [h.temperature_mean for h in daily_histories.histories]
                self._ax.plot(x, y, label=self.__label(daily_histories.location.name, 'Mean'))


class PrecipitationGraph(BaseGraph):
    """Creates the Precipitation history graph."""

    def __init__(self, precipitation_type: PrecipitationType, locations_daily_histories: List[PyDailyHistories],
                 font_size: FontSize):
        self._precipitation_type = precipitation_type
        self._locations_daily_histories = locations_daily_histories

        # the graph descriptions
        ylabel = 'Inches' if precipitation_type == PrecipitationType.RAIN else 'Percent'
        if precipitation_type == PrecipitationType.RAIN:
            title = 'Rain Amount'
        elif precipitation_type == PrecipitationType.HUMIDITY:
            title = 'Humidity'
        elif precipitation_type == PrecipitationType.CLOUD_COVER:
            title = 'Cloud Cover'
        else:
            title = 'Unknown'

        # create the plots
        BaseGraph.__init__(self, f'{title} Precipitation', ylabel, font_size,
                           include_legend=len(locations_daily_histories) > 1)

    def add_plots(self):
        """Called by the base class to create the Precipitation plots."""
        percent = lambda v: (v * 100.0) if v else 0.0
        for daily_histories in self._locations_daily_histories:
            x = _get_plot_dates(daily_histories)
            if self._precipitation_type == PrecipitationType.CLOUD_COVER:
                y = [percent(h.cloud_cover) for h in daily_histories.histories]
                self._ax.plot(x, y, label=daily_histories.location.name)
            elif self._precipitation_type == PrecipitationType.HUMIDITY:
                y = [percent(h.humidity) for h in daily_histories.histories]
                self._ax.plot(x, y, label=daily_histories.location.name)
            else:
                y = [h.precipitation_amount for h in daily_histories.histories]
                self._ax.plot(x, y, label=daily_histories.location.name)


class ConditionsGraph(BaseGraph):
    """Creates the Conditions history graph."""

    def __init__(self, conditions_type: ConditionsType, locations_daily_histories: List[PyDailyHistories],
                 font_size: FontSize):
        self._conditions_type = conditions_type
        self._locations_daily_histories = locations_daily_histories

        ylabel = 'UV Scale' if conditions_type == ConditionsType.UV_INDEX else 'MPH'
        plot_types = []
        if conditions_type & ConditionsType.UV_INDEX:
            plot_types.append('UV Index')
        if conditions_type & ConditionsType.WIND_SPEED:
            plot_types.append('Wind Speed')
        if conditions_type & ConditionsType.WIND_GUST:
            plot_types.append('Wind Gust')
        plot_count = len(plot_types)
        if plot_count == 1:
            title = plot_types[0]
        elif plot_count > 1:
            title = ' and '.join(plot_types)
        else:
            title = 'Unknown'

        # set up the label maker
        multiple_locations = len(locations_daily_histories) > 1
        multiple_plots = len(plot_types) > 1
        if not multiple_locations:
            self.__label = lambda _, w: w
        else:
            self.__label = lambda n, w: f'{n} {w}' if multiple_plots else n

        # create the plots
        BaseGraph.__init__(self, f'{title} Conditions', ylabel, font_size,
                           include_legend=multiple_locations or multiple_plots)

    def add_plots(self):
        """Called by the base class to create the Conditions plots."""
        mph = lambda v: v if v else 0
        for daily_histories in self._locations_daily_histories:
            x = _get_plot_dates(daily_histories)
            if self._conditions_type & ConditionsType.WIND_SPEED:
                y = [mph(h.wind_speed) for h in daily_histories.histories]
                self._ax.plot(x, y, label=self.__label(daily_histories.location.name, 'Wind Speed'))
            if self._conditions_type & ConditionsType.WIND_GUST:
                y = [mph(h.wind_gust) for h in daily_histories.histories]
                self._ax.plot(x, y, label=self.__label(daily_histories.location.name, 'Wind Gust'))
            if self._conditions_type & ConditionsType.UV_INDEX:
                y = [h.uv_index for h in daily_histories.histories]
                self._ax.plot(x, y, label=daily_histories.location.name)
