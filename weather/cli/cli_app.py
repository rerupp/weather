import argparse
import logging as log

from typing import Dict

from .commands import (
    CommandExec,
    ListPropertiesCMD,
    AddLocationCMD, ListLocationsCMD,
    AddWeatherHistoryCMD, ListWeatherCMD, ReportWeatherHistoryCMD,
    RemoveWeatherDataCMD
)
from weather.configuration import init_logging
from weather.domain import WeatherData


class WeatherCLI:

    def __init__(self):
        self._parser = argparse.ArgumentParser(prog="wcli")
        self._parser.add_argument("-D", "--debug", dest="debug", default=False, action='store_true',
                                  help="Display debug information.")
        self._parser.add_argument("-d", "--data", dest="data", metavar="DATA",
                                  help="The directory containing weather data (default={})."
                                  .format(WeatherData.WEATHER_DATA_DIR))

        self._dispatcher: Dict[str, CommandExec] = dict()
        cmd_parser = self._parser.add_subparsers(dest='cmd')
        commands = [ListLocationsCMD("ll"),
                    ListPropertiesCMD("ls"),
                    AddLocationCMD("al"),
                    ListWeatherCMD("lh"),
                    AddWeatherHistoryCMD("ah"),
                    ReportWeatherHistoryCMD("rh"),
                    RemoveWeatherDataCMD("del")]
        for command in commands:
            command.add_to_parser(cmd_parser)
            self._dispatcher[command.name] = command.execute

    def execute(self, args=None):
        options = self._parser.parse_args(args)
        weather_data = WeatherData(options.data) if options.data else WeatherData()

        if options.debug:
            log.getLogger().setLevel(log.DEBUG)
        log.debug("%s", options)

        if options.cmd:
            try:
                self._dispatcher[options.cmd](options, weather_data)
            except Exception as error:
                log.exception("Error executing '%s': %s...", options.cmd, str(error), exc_info=error)
        else:
            self._parser.print_usage()


def run_cli():
    init_logging()
    WeatherCLI().execute()
