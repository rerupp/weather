import argparse
import logging as log
from sys import stderr

from py_weather_lib import PyLocationFilters, PyWeatherConfig, PyWeatherData, create
from textual.app import App, ComposeResult
from textual.containers import VerticalScroll
from textual.widgets import Footer, Header

from .history_view import HistoryView


def run():
    # parse the command line
    parser = argparse.ArgumentParser(prog="tui")
    parser.add_argument('-v', '--verbose', default=0, action='count',
                        help="Log verbosity (default=WARN, v=INFO, vv=DEBUG, vvv+=TRACE).")
    parser.add_argument('-d', '--dir', metavar="DIR", default='weather_dir',
                        help="The weather data directory (default weather_data).")
    parser.add_argument('-l', '--log', metavar="FILE", default="tui.log",
                        help="The file where log output will be written.")
    parser.add_argument('-a', '--append', action='store_true',
                        help="When used log output will be appended.")
    options = parser.parse_args()

    match options.verbose:
        case 0:
            log_level = log.WARN
        case 1:
            log_level = log.INFO
        case _:
            log_level = log.DEBUG

    log.basicConfig(
        filename=options.log,
        filemode='a' if options.append else 'w',
        format='%(asctime)s %(module)s[%(lineno)d]: %(message)s',
        datefmt='%H:%M:%S',
        level=log_level
    )

    try:
        weather_data = create(PyWeatherConfig(dirname=options.dir, logfile='weather.log', log_level=options.verbose))
    except SystemError as error:
        print(f'There was an error creating the backend: {error!r}', file=stderr)
        exit(1)

    HistoryApp(weather_data).run()


class HistoryApp(App):
    ENABLE_COMMAND_PALETTE = False

    def __init__(self, weather_data: PyWeatherData):
        super().__init__()
        self.title = "Weather Histories"
        self._weather_data = weather_data

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        yield Footer()
        with VerticalScroll():
            for history_dates in self._weather_data.get_history_dates(PyLocationFilters([])):
                yield HistoryView(self._weather_data, history_dates)
